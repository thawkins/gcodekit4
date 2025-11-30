# Release 0.57.0-alpha.0

## Fixed
- **Windows Installer**: Fixed MSI installer error 2819 by adding the required `WIXUI_INSTALLDIR` property to bind the directory chooser dialog to `APPLICATIONFOLDER`. This resolves the "Control Folder on dialog InstallDirDlg needs a property linked to it" error that prevented installation.
