## ADDED Requirements

### Requirement: File Chooser Handling

The system SHALL support intercepting browser file chooser dialogs to programmatically set file inputs.

#### Scenario: File chooser detection

- **WHEN** a page action triggers a file input dialog
- **THEN** the system SHALL detect the file chooser via CDP `Page.fileChooserOpened` event
- **AND** SHALL route to `page.on('filechooser')` handlers
- **AND** SHALL create a `FileChooser` object

#### Scenario: Setting files via file chooser

- **WHEN** `fileChooser.setFiles(['./test-data/upload.pdf'])` is called
- **THEN** the system SHALL set the file input's value to the specified files via CDP `DOM.setFileInputFiles`
- **AND** SHALL resolve the file chooser dialog

#### Scenario: Multiple file selection

- **WHEN** `fileChooser.setFiles(['./photo1.jpg', './photo2.jpg', './photo3.jpg'])` is called on a multiple-file input
- **THEN** the system SHALL set all specified files on the input element
- **AND** the page SHALL receive all files in the `FileList`

#### Scenario: File chooser event with timeout

- **WHEN** `page.waitForEvent('filechooser', { timeout: 5000 })` is called
- **THEN** the system SHALL wait up to 5 seconds for a file chooser dialog
- **AND** SHALL throw a timeout error if no dialog appears
