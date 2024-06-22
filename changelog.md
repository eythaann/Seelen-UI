# Changelog

## [Unreleased]
### enhancements
- separate `information` and `developer tools` tabs in the settings.
- add a option to open the install path in explorer.
- focus settings window if already exist.
- better performance on canceling changes in settings.

### fix
- ahk error on save.

## [1.7.0]
### features
- add Network toolbar module.
- add WLAN selector to the Network toolbar module.
- add css variable (--config-accent-color-rgb) to be used with css functions like `rgb` or `rgba`.

### enhancements
- now placeholders, layouts and themes can be loaded from data users folder (`AppData\Roaming\com.seelen.seelen-ui`)
- now buttons and others components will use the user accent color.

### fix
- no max size on System Tray List module.

## [1.6.4]
### fix
- xbox games showing missing icons on SeelenWeg.

### enhancements
- follow user accent color for tray list and windows borders

### fix
- no showing promoted (pinned on taskbar) tray icons.
- toolbar no initialized correctly sometimes, now will retry if fails.
- battery no updating level.
- battery showing as always charging on default toolbar templates.
- tray overflow module no working on different languages.

### refactor
- refactor on window_api and AppBar structures.

## [1.6.3]
### enhancements
- only show a progress bar on update and not the complete installer GUI.

### fix
- main app no running if the forced creation of tray overflow fails.

## [1.6.2]
### features
- now `batteries` and `battery` (same as: `batteries[0]`) are available on the scope of power toolbar module.

### enhancements
- add battery crate to handle batteries info directly from their drivers.
- show if is smart charging.
- now battery module wont be shown if batteries are not found.

### fix
- battery showing 255%.

## [1.6.1]
### fix
- tray icons not showing on startup
- hidden trays if icon was not found (now will show a missing icon)

## [1.6.0]
### features
- add "Run as admin" option at context menu on Seelenweg. 
- allow receive commands using TCP connections.
- Add System Tray Icons module, (incomplete, devices like usb or windows antivirus trays are still not supported).

### enhancements
- improve power (battery) events performance.
- Window manager disabled by default to new users.

### refactor
- remove tauri single instance plugin by TCP connection.

## [1.5.0]
### features
- new placeholder added to the bundle as alternative to default.
- new workspace item available to be used in placeholders.

### enhancements
- support fullscreen apps (will hide the toolbar and the weg on fullscreen an app).

### fix
- showing incorrect format on dates at start of the app.
- complex text with icons on toolbar items cause wraps.
- missing icons on some uwp apps.

### refactor
- refactor on window event manager to allow synthetic events.

## [1.4.1]
### fix
- no truncating text on toolbar items overflow.
- rendering empty items on toolbar when empty placeholder is declared.

## [1.4.0]
### features
- Modular Themes
- Themes now allow tags to be categorized.
- Allow add, organize, combine multiple themes as cascade layers.
- Themes now allow folder structure to improve developers experience.

### refactor
- Now themes will use .yml files instead json to improve developers experience.
- Themes schema updated, no backwards compatibility with json themes. (.json in themes folder will be ignored)

### fix
- No hiding the taskbar correctly.

## [1.3.4]
### enhancements
- Add splash screen to Settings window.
- Add discord link on Information Section.

### refactor
- Use TaskScheduler for autostart Seelen with priority and admin privileges.

### fix
- bad zorder on Weg and Toolbar under the WM borders

## [1.3.3]
### features
- Multi-monitor support for Fancy Toolbar.
- Multi-monitor support for Seelenweg.

## [1.3.2]
### enhancements
- Remove unnecessary tooltip collision on toolbar items.

### fix
- Crash on restoring app in other virtual desktop using Weg.
- Touch events not working on Toolbar and Weg.

## [1.3.1]
### fix
- disable binding monitors and monitors on apps configurations for now.

## [1.3.0]
### features
- Allow pin apps on Open using Apps Configurations.
- Allow changes Shortcuts using UI.
- Allow Binary Conditions in Apps Configurations Identifiers.
- Allow change the Auto hide behavior for Seelenweg.

### enhancements
- Close AHK by itself if app is crashed or forcedly closed.
- Configurations by apps are enabled again.
- Allow open settings file from Extras/Information
- Add opacity to toolbar (theme: default)

### fix
- Ahk not closing on app close or when user change options.

## [1.2.4]
### enhancements
- Automatic MSIX bundle.
- Add Github Actions for Releases.
- Add Github Actions for Web Page.

## [1.2.3]
### features
- Allow customize Fancy Toolbar modules using placeholders yaml files.
- Add fast settings for toolbar allowing to adjust volume, brightness, etc.

## [1.2.2]
### enhancements
- if app on weg is cloak, change of virtual desktop instead minimize/restore

### fix
- no closing AHK instances
- floating size on fallback
- reservation not working properly
- ignore top most windows by default (normally these are tools or widgets)
- minimization on weg not working properly if window manager is activated
- change focus using commands not working with conditional layouts
- randomly frozen app on start
- no tiling UWP apps

## [1.2.1]
### enhancements
- Allow quit from settings
- Using Box-Content style in the position of windows instead outlined for a better user experience

### fix
- Managing windows without caption (Title bar)
- can't update border configurations
- hiding dock on switching virtual desktops

## [1.2.0]
### fix
- Taskbar showing instead be always hidden

## [1.1.1]
### fix
- Bad download URL in Update Notification
- Showing update notification on installations by Windows Store

## [1.1.0]
### features
- Add Smart Auto Hide for Seelenweg.
- Add visible Separators Option
- Enable animations for items into LEFT, TOP, RIGHT positions

### enhancements
- Now the copy handles option will return hexadecimal handles instead decimal (good for faster debug in tools like spy++).

### fix
- duped handles
- inconsistencies in separators width

## [1.0.1]
### fix
- App downloaded form Microsoft Store was not running without admin.

## [1.0.0]
### refactored
- Update notifications always enabled for nsis installer
- Update notifications will not appear if app is installed using msix (Microsoft Store).

### enhancements
- Now by default if user is Admin, UAC will be triggered on run the app to allow a better integrated experience in SeelenWeg and Komorebi Tiling Manager.

## [1.0.0-prerelease.14]
### features
- add indicator to know opens and focused apps on SeelenWeg
- allow set the position of seelenweg (left, top, right, bottom) 🎉

### enhancements
- only creates app icons the first time they are loaded

### refactor
- change themes implementation to allow customs css files

### fix
- incorrect icon for UWP (was using store icon instead taskbar icon)
- replacing icons on each load
- showing logs of opened apps on development
- offset margins working like windows RECT instead like one side margins

## [1.0.0-prerelease.13]
### features
- add Themes Feature 🎉 (incomplete only for Seelenweg for now)
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
- add a update tab to allow users decide if will receive notifications for updates

## [1.0.0-prerelease.9]
## [1.0.0-prerelease.8]
- add functionality to pause btn on tray menu

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
### enhancements
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
- missing property on schema
- white screen on start app

## [1.0.0-beta.10]
### added
- add a new way to match applications by path

### fixed
- searching feature on apps
- no focusing windows on change workspace
- autostacking not working properly
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
- not updating the path of installation folder on update for windows tasks

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
- delete bound monitor and workspace on an application
- bad installation on setup