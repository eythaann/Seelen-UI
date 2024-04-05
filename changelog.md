# Changelog

## [Unreleased]
### features
- add indicator to know opens and focused apps on SeelenWeg

## [1.0.0-prerelease.13]
### features
- add Themes Feature 🎉 (incompleted only for Seelenweg for now)
- add SeelenWeg (a Dock or Taskbar) beta feature
- add SeelenWeg in to Settings
- add ContextMenu for apps in SeelenWeg
- allow move apps in the Weg 😄
- add Grouped Apps in one item
- live reload of Apps on events like change of title
- UWP apps support

### enhancements
- move readme blob to documentation/images

## [1.0.0-prerelease.12]
### enhancements
- add some traces on functions to save logs

### fix
- clean installation of komorebi no working

## [1.0.0-prerelease.11]
### refactor
- little improvements on background code

### fix
- initial users can not save the settings

## [1.0.0-prerelease.10]
### features
- add a update tab to allow users decide if will recive notifications for updates

## [1.0.0-prerelease.9]
## [1.0.0-prerelease.8]
- add funcionality to pause btn on tray menu

## [1.0.0-prerelease.6]
### added
- Enable Updater 🎉

## [1.0.0-prerelease.3]
### fix
- icon not showing on tray
- poor icon quality on task bar
- StartUp running bad exe file

## [1.0.0-prerelease.2]
## [1.0.0-prerelease.1]
### added
- implement tray icon

### refactored
- Migrate all app background from Electron ⚡ to Tauri 🦀
- reimplement startup to use native system startup
- reimplement included shortcuts with ahk
- reimplement komorebi autostart
- reimplement installer to use NSIS
- refactor folder structure to isolate front-end apps

## [1.0.0-beta.13]
### ehancements
- improve maximized windows experience

### fixed
- fix resize not working (now works like master)

## [1.0.0-beta.12]
### added
- show current used versions on information
- add grid layout preview
- add win + k to open komorebi settings

### refactored
- update komorebi to 0.1.22

### removed
- remove invisible borders feature

## [1.0.0-beta.11]
### fixed
- missing propety on schema
- white screen on start app

## [1.0.0-beta.10]
### added
- add a new way to match applications by path

### fixed
- searching feature on apps
- no focusing windows on change workspace
- autostacking not working propertly
- workspaces rules not working

## [1.0.0-beta.9]
### added
- add popups on actions 🦀
- now can switch from installed and packaged and should work as the same

### fixed
- fix no removing old path
- lag on many applications

## [1.0.0-beta.8]
### added
- add more templates

## [1.0.0-beta.7]
### fixed
- fix first install

## [1.0.0-beta.6]
### added
- delete old paths on update

### fixed
- fix not saving templates
- fix toggle ahk shortcuts does not run or stop the instance
- running ahk when disabled
- not updating the path of intallation folder on update for windows tasks

## [1.0.0-beta.5]
### added
- new searching option for applications
- templates feature

### fixed
- including ghost apps on migration

## [1.0.0-beta.4]
### added
- new feature of invisible borders per app
- new easy way to hard restart the services and AHK

### changed
- delete border overflow and changed for invisible borders per app

### fixed
- components was not triggering dark mode correctly

## [1.0.0-beta.3]
### added
- new apps templates
- add AHK as a dependency to show to new users
- add AHK settings

## [1.0.0-beta.2]
### added
- export option for apps

### fixed
- delete binded monitor and workpace on an application
- bad intallation on setup