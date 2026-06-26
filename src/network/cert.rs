use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rcgen::{CertificateParams, DistinguishedName, DnType, IsCa, BasicConstraints, KeyPair, KeyUsagePurpose, Issuer, date_time_ymd};

pub struct CertStore {
    ca_cert_pem: String,
    ca_cert_der: Vec<u8>,
    ca_params: CertificateParams,
    ca_key_pem: String,
    domain_certs: Mutex<HashMap<String, Arc<DomainCert>>>,
}

struct DomainCert {
    cert_der: Vec<u8>,
    key_pem: String,
}

impl CertStore {
    pub fn new() -> Result<Self, String> {
        let ca_key = KeyPair::generate().map_err(|e| format!("Key generation failed: {}", e))?;
        let ca_key_pem = ca_key.serialize_pem();

        let mut ca_params = CertificateParams::default();
        ca_params.distinguished_name = DistinguishedName::new();
        ca_params.distinguished_name.push(DnType::CommonName, "TurboSheet MitM CA");
        ca_params.distinguished_name.push(DnType::OrganizationName, "TurboSheet");
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];
        ca_params.not_before = date_time_ymd(2025, 1, 1);
        ca_params.not_after = date_time_ymd(2035, 1, 1);

        let ca_cert = ca_params
            .self_signed(&ca_key)
            .map_err(|e| format!("CA self-sign failed: {}", e))?;
        let ca_cert_pem = ca_cert.pem();
        let ca_cert_der = ca_cert.der().to_vec();

        Ok(Self {
            ca_cert_pem,
            ca_cert_der,
            ca_params,
            ca_key_pem,
            domain_certs: Mutex::new(HashMap::new()),
        })
    }

    pub fn ca_cert_der(&self) -> &[u8] {
        &self.ca_cert_der
    }

    pub fn ca_cert_pem(&self) -> &str {
        &self.ca_cert_pem
    }

    pub fn cert_for_domain(&self, domain: &str) -> Result<(Vec<u8>, String), String> {
        {
            let cache = self.domain_certs.lock().unwrap();
            if let Some(existing) = cache.get(domain) {
                return Ok((existing.cert_der.clone(), existing.key_pem.clone()));
            }
        }

        let key = KeyPair::generate().map_err(|e| format!("Domain key generation failed: {}", e))?;
        let key_pem = key.serialize_pem();

        let mut params = CertificateParams::new(vec![domain.to_string()])
            .map_err(|e| format!("Invalid domain '{}': {}", domain, e))?;
        params.distinguished_name = DistinguishedName::new();
        params.distinguished_name.push(DnType::CommonName, domain);
        params.is_ca = IsCa::NoCa;
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        params.not_before = date_time_ymd(2025, 1, 1);
        params.not_after = date_time_ymd(2026, 1, 1);

        let ca_key = KeyPair::from_pem(&self.ca_key_pem)
            .map_err(|e| format!("CA key parse failed: {}", e))?;
        let issuer = Issuer::new(self.ca_params.clone(), ca_key);

        let cert = params
            .signed_by(&key, &issuer)
            .map_err(|e| format!("Domain cert signing failed: {}", e))?;

        let cert_der = cert.der().to_vec();
        let domain_cert = Arc::new(DomainCert {
            cert_der: cert_der.clone(),
            key_pem: key_pem.clone(),
        });

        self.domain_certs.lock().unwrap().insert(domain.to_string(), domain_cert);
        Ok((cert_der, key_pem))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ca_generates_and_self_signs() {
        let store = CertStore::new().unwrap();
        assert!(store.ca_cert_pem().starts_with("-----BEGIN CERTIFICATE-----"));
        assert!(!store.ca_cert_der().is_empty());
    }

    #[test]
    fn domain_cert_is_signed_by_ca() {
        let store = CertStore::new().unwrap();
        let (cert_der, key_pem) = store.cert_for_domain("example.com").unwrap();
        assert!(!cert_der.is_empty());
        assert!(key_pem.contains("BEGIN PRIVATE KEY"));

        let (cert_der2, _) = store.cert_for_domain("example.com").unwrap();
        assert_eq!(cert_der, cert_der2);
    }

    #[test]
    fn different_domains_get_different_certs() {
        let store = CertStore::new().unwrap();
        let (cert1, _) = store.cert_for_domain("first.example.com").unwrap();
        let (cert2, _) = store.cert_for_domain("second.example.com").unwrap();
        assert_ne!(cert1, cert2);
    }
}
