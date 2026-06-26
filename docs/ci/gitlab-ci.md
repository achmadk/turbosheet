# GitLab CI Integration

Complete guide for running TurboSheet tests in GitLab CI.

## Basic Setup

Create `.gitlab-ci.yml`:

```yaml
stages:
  - test

variables:
  TURBO_SHEET_VERSION: "0.1.0"
  NODE_VERSION: "20"

cache:
  key: ${CI_COMMIT_REF_SLUG}
  paths:
    - node_modules/
    - .cache/tsheet/

.test_template: &test_template
  image: node:${NODE_VERSION}
  before_script:
    - npm ci
    - npx tsheet install
  script:
    - npx tsheet test --reporter=list
  artifacts:
    when: always
    paths:
      - test-results/
    expire_in: 7 days

test:
  <<: *test_template
```

## With Multiple Browsers

```yaml
stages:
  - test

variables:
  NODE_VERSION: "20"

test:chromium:
  image: node:${NODE_VERSION}
  stage: test
  cache:
    key: ${CI_COMMIT_REF_SLUG}
    paths:
      - node_modules/
      - .cache/tsheet/
  before_script:
    - npm ci
    - npx tsheet install chromium
  script:
    - npx tsheet test --browser=chromium --reporter=list
  artifacts:
    when: always
    paths:
      - test-results/
    expire_in: 7 days

test:firefox:
  image: node:${NODE_VERSION}
  stage: test
  cache:
    key: ${CI_COMMIT_REF_SLUG}
    paths:
      - node_modules/
      - .cache/tsheet/
  before_script:
    - npm ci
    - npx tsheet install firefox
  script:
    - npx tsheet test --browser=firefox --reporter=list
  artifacts:
    when: always
    paths:
      - test-results/
    expire_in: 7 days

test:webkit:
  image: node:${NODE_VERSION}
  stage: test
  cache:
    key: ${CI_COMMIT_REF_SLUG}
    paths:
      - node_modules/
      - .cache/tsheet/
  before_script:
    - npm ci
    - npx tsheet install webkit
  script:
    - npx tsheet test --browser=webkit --reporter=list
  artifacts:
    when: always
    paths:
      - test-results/
    expire_in: 7 days
```

## With Sharded Tests

```yaml
stages:
  - test

variables:
  NODE_VERSION: "20"
  SHARD_TOTAL: 4

.test_template: &test_template
  image: node:${NODE_VERSION}
  cache:
    key: ${CI_COMMIT_REF_SLUG}
    paths:
      - node_modules/
      - .cache/tsheet/
  before_script:
    - npm ci
    - npx tsheet install
  script:
    - npx tsheet test --shard=$CI_NODE_INDEX/$SHARD_TOTAL --reporter=list
  artifacts:
    when: always
    paths:
      - test-results/
    expire_in: 7 days

test:shard:
  stage: test
  parallel:
    matrix:
      - CI_NODE_INDEX: [1, 2, 3, 4]
  <<: *test_template
```

## GitLab CI with Review Apps

```yaml
stages:
  - deploy
  - test

variables:
  NODE_VERSION: "20"

review_app:
  stage: deploy
  image: node:${NODE_VERSION}
  script:
    - npm ci
    - npm run build
    - npm run start &
    - sleep 5
    - echo "Review app deployed"
  environment:
    name: review/$CI_COMMIT_REF_NAME
    url: https://review-$CI_COMMIT_REF_SLUG.example.com
  only:
    - branches-except-main

e2e_test:
  stage: test
  image: node:${NODE_VERSION}
  cache:
    key: ${CI_COMMIT_REF_SLUG}
    paths:
      - node_modules/
      - .cache/tsheet/
  before_script:
    - npm ci
    - npx tsheet install
  script:
    - npx tsheet test --reporter=list
  artifacts:
    when: always
    paths:
      - test-results/
    expire_in: 7 days
  dependencies:
    - review_app
  environment:
    name: review/$CI_COMMIT_REF_NAME
  only:
    - branches-except-main
```

## GitLab CI with JUnit XML

```yaml
stages:
  - test

variables:
  NODE_VERSION: "20"

test:
  image: node:${NODE_VERSION}
  cache:
    key: ${CI_COMMIT_REF_SLUG}
    paths:
      - node_modules/
      - .cache/tsheet/
  before_script:
    - npm ci
    - npx tsheet install
  script:
    - npx tsheet test --reporter=junit --output-file=test-results/junit.xml
  artifacts:
    when: always
    reports:
      junit: test-results/junit.xml
    paths:
      - test-results/
    expire_in: 7 days
```

## Complete Example

```yaml
stages:
  - install
  - test

variables:
  NODE_VERSION: "20"

cache:
  key: ${CI_COMMIT_REF_SLUG}
  paths:
    - node_modules/
    - .cache/tsheet/

install:
  stage: install
  image: node:${NODE_VERSION}
  script:
    - npm ci
  artifacts:
    paths:
      - node_modules/

test:chromium:
  stage: test
  image: node:${NODE_VERSION}
  cache:
    key: ${CI_COMMIT_REF_SLUG}
    paths:
      - node_modules/
      - .cache/tsheet/
  before_script:
    - npx tsheet install chromium
  script:
    - npx tsheet test --browser=chromium --reporter=list
  artifacts:
    when: always
    paths:
      - test-results/
    expire_in: 7 days

test:firefox:
  stage: test
  image: node:${NODE_VERSION}
  cache:
    key: ${CI_COMMIT_REF_SLUG}
    paths:
      - node_modules/
      - .cache/tsheet/
  before_script:
    - npx tsheet install firefox
  script:
    - npx tsheet test --browser=firefox --reporter=list
  artifacts:
    when: always
    paths:
      - test-results/
    expire_in: 7 days

test:sharded:
  stage: test
  image: node:${NODE_VERSION}
  parallel:
    matrix:
      - SHARD: [1, 2, 3, 4]
  cache:
    key: ${CI_COMMIT_REF_SLUG}
    paths:
      - node_modules/
      - .cache/tsheet/
  before_script:
    - npx tsheet install
  script:
    - npx tsheet test --shard=$SHARD/4 --reporter=list
  artifacts:
    when: always
    paths:
      - test-results/
    expire_in: 7 days
```
