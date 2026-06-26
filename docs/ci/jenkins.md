# Jenkins Integration

Complete guide for running TurboSheet tests in Jenkins CI/CD.

## Prerequisites

- Jenkins with Docker or Node.js installed
- Pipeline plugin installed

## Declarative Pipeline

Create `Jenkinsfile` in your repository:

```groovy
pipeline {
    agent any

    environment {
        NODE_VERSION = '20'
        TURBO_SHEET_VERSION = '0.1.0'
    }

    stages {
        stage('Install') {
            steps {
                sh '''
                    nvm use ${NODE_VERSION}
                    npm ci
                    npx tsheet install
                '''
            }
        }

        stage('Test') {
            steps {
                sh 'npx tsheet test --reporter=list'
            }
            post {
                always {
                    junit 'test-results/*.xml'
                    archiveArtifacts artifacts: 'test-results/**', fingerprint: true
                }
            }
        }
    }

    post {
        always {
            cleanWs()
        }
    }
}
```

## With Docker Agent

```groovy
pipeline {
    agent {
        docker {
            image 'node:20'
            args '-u root:root'
        }
    }

    stages {
        stage('Install') {
            steps {
                sh 'npm ci'
                sh 'npx tsheet install'
            }
        }

        stage('Test') {
            steps {
                sh 'npx tsheet test --reporter=junit --output-file=test-results/junit.xml'
            }
            post {
                always {
                    junit 'test-results/*.xml'
                    archiveArtifacts artifacts: 'test-results/**', fingerprint: true
                }
            }
        }
    }
}
```

## With Multiple Browsers

```groovy
pipeline {
    agent any

    stages {
        stage('Install') {
            steps {
                sh '''
                    npm ci
                    npx tsheet install chromium
                    npx tsheet install firefox
                '''
            }
        }

        stage('Test - Chromium') {
            steps {
                sh 'npx tsheet test --browser=chromium --reporter=junit --output-file=test-results/chromium-junit.xml'
            }
            post {
                always {
                    junit 'test-results/chromium-*.xml'
                }
            }
        }

        stage('Test - Firefox') {
            steps {
                sh 'npx tsheet test --browser=firefox --reporter=junit --output-file=test-results/firefox-junit.xml'
            }
            post {
                always {
                    junit 'test-results/firefox-*.xml'
                }
            }
        }
    }
}
```

## With Sharded Tests

```groovy
pipeline {
    agent any

    environment {
        SHARD_TOTAL = '4'
    }

    stages {
        stage('Install') {
            steps {
                sh '''
                    npm ci
                    npx tsheet install
                '''
            }
        }

        stage('Test - Sharded') {
            steps {
                script {
                    def tests = [:]
                    for (int i = 1; i <= SHARD_TOTAL.toInteger(); i++) {
                        def shard = i
                        tests["Shard ${shard}"] = {
                            sh "npx tsheet test --shard=${shard}/${SHARD_TOTAL} --reporter=junit --output-file=test-results/shard-${shard}-junit.xml"
                        }
                    }
                    parallel tests
                }
            }
            post {
                always {
                    junit 'test-results/shard-*-junit.xml'
                    archiveArtifacts artifacts: 'test-results/**', fingerprint: true
                }
            }
        }
    }
}
```

## Jenkinsfile with Browser Caching

```groovy
pipeline {
    agent any

    environment {
        NODE_VERSION = '20'
        BROWSER_CACHE = '${WORKSPACE}/.cache/tsheet'
    }

    stages {
        stage('Setup') {
            steps {
                sh '''
                    mkdir -p ${BROWSER_CACHE}
                    nvm use ${NODE_VERSION}
                    npm ci
                '''
            }
        }

        stage('Install Browsers') {
            steps {
                sh 'npx tsheet install chromium'
            }
        }

        stage('Test') {
            steps {
                sh 'npx tsheet test --reporter=junit --output-file=test-results/junit.xml'
            }
            post {
                always {
                    junit 'test-results/*.xml'
                    archiveArtifacts artifacts: 'test-results/**', fingerprint: true
                }
            }
        }
    }

    post {
        always {
            cleanWs()
        }
    }
}
```

## Jenkins Shared Library

Create a shared library at `vars/turboSheet.groovy`:

```groovy
def call(Map config = [:]) {
    def browser = config.browser ?: 'chromium'
    def reporter = config.reporter ?: 'list'
    def outputFile = config.outputFile ?: 'test-results/junit.xml'

    pipeline {
        agent any

        stages {
            stage('Install') {
                steps {
                    sh '''
                        npm ci
                        npx tsheet install ${browser}
                    '''
                }
            }

            stage('Test') {
                steps {
                    sh "npx tsheet test --browser=${browser} --reporter=${reporter} --output-file=${outputFile}"
                }
                post {
                    always {
                        junit outputFile
                        archiveArtifacts artifacts: 'test-results/**', fingerprint: true
                    }
                }
            }
        }
    }
}
```

Usage in Jenkinsfile:

```groovy
@Library('turbosheet-library') _

turboSheet browser: 'chromium', reporter: 'list'
```

## Complete Jenkinsfile Example

```groovy
pipeline {
    agent {
        docker {
            image 'node:20'
            args '-u root:root -v /cache/tsheet:/root/.cache/tsheet'
        }
    }

    environment {
        NODE_VERSION = '20'
        TURBO_SHEET_VERSION = '0.1.0'
        BROWSERS = 'chromium,firefox'
    }

    options {
        timeout(time: 30, unit: 'MINUTES')
        buildDiscarder(logRotator(numToKeepStr: '10'))
    }

    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }

        stage('Install Dependencies') {
            steps {
                sh '''
                    npm ci
                '''
            }
        }

        stage('Install Browsers') {
            steps {
                sh '''
                    npx tsheet install chromium
                '''
            }
        }

        stage('Lint') {
            steps {
                sh 'npm run lint || true'
            }
        }

        stage('Test - Chromium') {
            steps {
                sh 'npx tsheet test --browser=chromium --reporter=junit --output-file=test-results/chromium-junit.xml'
            }
            post {
                always {
                    junit 'test-results/chromium-junit.xml'
                    publishHTML([
                        reportDir: 'test-results',
                        reportFiles: 'report.html',
                        reportName: 'HTML Report'
                    ])
                }
            }
        }

        stage('Test - Firefox') {
            steps {
                sh 'npx tsheet install firefox'
                sh 'npx tsheet test --browser=firefox --reporter=junit --output-file=test-results/firefox-junit.xml'
            }
            post {
                always {
                    junit 'test-results/firefox-junit.xml'
                }
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'test-results/**', fingerprint: true
            cleanWs()
        }
        failure {
            slackSend message: "Build ${env.BUILD_NUMBER} failed: ${env.BUILD_URL}"
        }
        success {
            slackSend message: "Build ${env.BUILD_NUMBER} passed: ${env.BUILD_URL}"
        }
    }
}
```
