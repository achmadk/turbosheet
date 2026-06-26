# CircleCI Integration

Complete guide for running TurboSheet tests in CircleCI.

## Basic Setup

Create `.circleci/config.yml`:

```yaml
version: 2.1

orbs:
  node: circleci/node@5

jobs:
  test:
    docker:
      - image: cimg/node:20
    steps:
      - checkout
      - node/install-packages:
          pkg-manager: npm
      - run:
          name: Install browsers
          command: npx tsheet install
      - run:
          name: Run tests
          command: npx tsheet test --reporter=list
      - store_test_results:
          path: test-results/
      - store_artifacts:
          path: test-results/

workflows:
  test:
    jobs:
      - test
```

## With Multiple Browsers

```yaml
version: 2.1

orbs:
  node: circleci/node@5

jobs:
  test-chromium:
    docker:
      - image: cimg/node:20
    steps:
      - checkout
      - node/install-packages:
          pkg-manager: npm
      - run:
          name: Install Chromium
          command: npx tsheet install chromium
      - run:
          name: Run tests (Chromium)
          command: npx tsheet test --browser=chromium --reporter=list
      - store_test_results:
          path: test-results/
      - store_artifacts:
          path: test-results/

  test-firefox:
    docker:
      - image: cimg/node:20
    steps:
      - checkout
      - node/install-packages:
          pkg-manager: npm
      - run:
          name: Install Firefox
          command: npx tsheet install firefox
      - run:
          name: Run tests (Firefox)
          command: npx tsheet test --browser=firefox --reporter=list
      - store_test_results:
          path: test-results/
      - store_artifacts:
          path: test-results/

  test-webkit:
    docker:
      - image: cimg/node:20
    steps:
      - checkout
      - node/install-packages:
          pkg-manager: npm
      - run:
          name: Install WebKit
          command: npx tsheet install webkit
      - run:
          name: Run tests (WebKit)
          command: npx tsheet test --browser=webkit --reporter=list
      - store_test_results:
          path: test-results/
      - store_artifacts:
          path: test-results/

workflows:
  test:
    jobs:
      - test-chromium
      - test-firefox
      - test-webkit
```

## With Sharded Tests

```yaml
version: 2.1

orbs:
  node: circleci/node@5

jobs:
  test:
    docker:
      - image: cimg/node:20
    parameters:
      shard:
        type: integer
      total:
        type: integer
    steps:
      - checkout
      - node/install-packages:
          pkg-manager: npm
      - run:
          name: Install browsers
          command: npx tsheet install
      - run:
          name: Run tests (shard << parameters.shard >>/<< parameters.total >>)
          command: npx tsheet test --shard=<< parameters.shard >>/<< parameters.total >> --reporter=junit --output-file=test-results/junit.xml
      - store_test_results:
          path: test-results/
      - store_artifacts:
          path: test-results/

workflows:
  test:
    jobs:
      - test:
          matrix:
            parameters:
              shard: [1, 2, 3, 4]
              total: [4]
```

## With Caching

```yaml
version: 2.1

orbs:
  node: circleci/node@5

commands:
  restore-browser-cache:
    steps:
      - restore_cache:
          keys:
            - v1-browsers-{{ arch }}-{{ checksum "package.json" }}
            - v1-browsers-{{ arch }}-
            - v1-browsers-

  save-browser-cache:
    steps:
      - save_cache:
          key: v1-browsers-{{ arch }}-{{ checksum "package.json" }}
          paths:
            - ~/.cache/tsheet

jobs:
  test:
    docker:
      - image: cimg/node:20
    steps:
      - checkout
      - node/install-packages:
          pkg-manager: npm
      - restore-browser-cache
      - run:
          name: Install browsers
          command: npx tsheet install
      - save-browser-cache
      - run:
          name: Run tests
          command: npx tsheet test --reporter=junit --output-file=test-results/junit.xml
      - store_test_results:
          path: test-results/
      - store_artifacts:
          path: test-results/

workflows:
  test:
    jobs:
      - test
```

## With Slack Notifications

```yaml
version: 2.1

orbs:
  node: circleci/node@5
  slack: circleci/slack@4

jobs:
  test:
    docker:
      - image: cimg/node:20
    steps:
      - checkout
      - node/install-packages:
          pkg-manager: npm
      - run:
          name: Install browsers
          command: npx tsheet install
      - run:
          name: Run tests
          command: npx tsheet test --reporter=junit --output-file=test-results/junit.xml
      - store_test_results:
          path: test-results/
      - store_artifacts:
          path: test-results/

workflows:
  test:
    jobs:
      - test:
          context: slack-notifications
```

## Complete Example

```yaml
version: 2.1

orbs:
  node: circleci/node@5

commands:
  restore-browser-cache:
    steps:
      - restore_cache:
          keys:
            - v1-browsers-{{ arch }}-{{ checksum "package.json" }}
            - v1-browsers-{{ arch }}-
            - v1-browsers-

  save-browser-cache:
    steps:
      - save_cache:
          key: v1-browsers-{{ arch }}-{{ checksum "package.json" }}
          paths:
            - ~/.cache/tsheet

  install-browsers:
    steps:
      - run:
          name: Install Chromium
          command: npx tsheet install chromium

  run-tests:
    steps:
      - run:
          name: Run tests
          command: npx tsheet test --browser=chromium --reporter=junit --output-file=test-results/junit.xml

jobs:
  lint:
    docker:
      - image: cimg/node:20
    steps:
      - checkout
      - node/install-packages:
          pkg-manager: npm
      - run:
          name: Lint
          command: npm run lint

  test:
    docker:
      - image: cimg/node:20
    steps:
      - checkout
      - node/install-packages:
          pkg-manager: npm
      - restore-browser-cache
      - install-browsers
      - save-browser-cache
      - run-tests
      - store_test_results:
          path: test-results/
      - store_artifacts:
          path: test-results/
          destination: test-results

  test-sharded:
    docker:
      - image: cimg/node:20
    parameters:
      shard:
        type: integer
    steps:
      - checkout
      - node/install-packages:
          pkg-manager: npm
      - restore-browser-cache
      - install-browsers
      - save-browser-cache
      - run:
          name: Run tests (shard << parameters.shard >>/4)
          command: npx tsheet test --browser=chromium --shard=<< parameters.shard >>/4 --reporter=junit --output-file=test-results/junit.xml
      - store_test_results:
          path: test-results/

workflows:
  version: 2
  test:
    jobs:
      - lint
      - test:
          requires:
            - lint
      - test-sharded:
          name: "test-shard-1"
          requires:
            - lint
          parameters:
            shard: 1
      - test-sharded:
          name: "test-shard-2"
          requires:
            - lint
          parameters:
            shard: 2
      - test-sharded:
          name: "test-shard-3"
          requires:
            - lint
          parameters:
            shard: 3
      - test-sharded:
          name: "test-shard-4"
          requires:
            - lint
          parameters:
            shard: 4
```
