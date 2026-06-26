## ADDED Requirements

### Requirement: Functional Network Proxy

The system SHALL implement a transparent HTTP/HTTPS proxy that correctly tunnels traffic, supports route interception, and captures request bodies.

#### Scenario: HTTPS CONNECT tunneling

- **WHEN** the browser sends a CONNECT request to the proxy
- **THEN** the proxy SHALL establish a bidirectional TCP tunnel between the browser and the target host
- **AND** SHALL transparently forward all data in both directions until either side disconnects
- **AND** SHALL clean up the tunnel resources on disconnection

#### Scenario: Route continue passthrough

- **WHEN** a route handler calls `route.continue()` without options
- **THEN** the proxy SHALL forward the original request to its destination
- **AND** SHALL return the destination's response to the browser
- **WHEN** `route.continue({ url, headers, postData })` is called with options
- **THEN** the proxy SHALL apply the URL rewrite, header modifications, and POST data override before forwarding

#### Scenario: POST body capture

- **WHEN** a POST request passes through the proxy
- **THEN** the proxy SHALL read and buffer the full request body from the `Incoming` stream
- **AND** `route.request().postData()` SHALL return the captured body bytes
- **AND** `post_data: None` SHALL never appear for requests with bodies

#### Scenario: MitM HTTPS interception

- **WHEN** proxy mode is active and HTTPS interception is enabled
- **THEN** the proxy SHALL generate a self-signed CA certificate on first launch
- **AND** SHALL generate per-domain certificates signed by the CA
- **AND** SHALL decrypt HTTPS traffic for route inspection and modification

#### Scenario: CDP Fetch mode (preferred)

- **WHEN** the browser is Chromium and CDP Fetch mode is enabled
- **THEN** the system SHALL use CDP `Fetch.enable` + `Fetch.requestPaused` instead of TCP proxy
- **AND** SHALL intercept HTTPS bodies natively without MitM CA
- **AND** SHALL intercept WebSocket connections via CDP events
