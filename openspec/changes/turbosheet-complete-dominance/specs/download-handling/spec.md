## ADDED Requirements

### Requirement: Download Handling

The system SHALL support interception and management of file downloads initiated by page actions.

#### Scenario: Download detection

- **WHEN** a page action triggers a file download
- **THEN** the system SHALL detect the download via CDP `Browser.downloadWillBegin` event
- **AND** SHALL create a `Download` object tracking the download state

#### Scenario: Getting download path

- **WHEN** `download.path()` is called on a completed download
- **THEN** the system SHALL return the file system path where the download was saved

#### Scenario: Saving download to custom location

- **WHEN** `download.saveAs('./reports/summary.pdf')` is called
- **THEN** the system SHALL move or copy the downloaded file to the specified path
- **AND** SHALL resolve when the file operation completes

#### Scenario: Canceling a download

- **WHEN** `download.cancel()` is called during an active download
- **THEN** the system SHALL abort the in-progress download via CDP
- **AND** SHALL clean up any partial file on disk

#### Scenario: Waiting for download event

- **WHEN** `page.waitForEvent('download')` is called before clicking a download link
- **THEN** the system SHALL wait up to the configured timeout for a download to begin
- **AND** SHALL resolve with the Download object

#### Scenario: Download progress tracking

- **WHEN** a download is in progress
- **THEN** the system SHALL subscribe to CDP `Browser.downloadProgress` events
- **AND** SHALL expose `download.url()`, `download.suggestedFilename()`, and download state
