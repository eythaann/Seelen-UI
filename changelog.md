# Changelog

## [Unreleased]
## [1.9.4]
### fix
- app crashing for new users

## [1.9.3]
### performance
- reduce load time from ~7s to ~4s

### features
- .slu and uri now are loaded correctly on seelen ui.
- allow change wallpaper from seelen settings.

### enhancements
- add file associations for .slu files
- add uri associations for seelen-ui:uri
- improve settings editor experience by adding live reload feature.

### fix
- cli no working on production

## [1.9.1]
### fix
- no listening window moving of virtual desktop events.
- no closing or starting widgets on settings changes.
- no listening monitors changes.
- no loading toolbar modules on wake up

## [1.9.0]
### features
- allow custom images on toolbar by `imgFromUrl`, `imgFromPath` and `imgFromExe` functions.
- add notifications module to toolbar.
- add exe path to window in generic module for toolbar.
- add focused window icon to default toolbar layouts.

### enhancements
- icons now are recreated from exe path if icon was deleted.
- uwp icons now are loaded from background.
- improvements on themes selector.
- improvements on system color detection and expose more system colors based in accent gamma.
- improve theme creation experience by adding live reload feature.
- improve toolbar layouts (placeholders) creation experience by adding live reload feature.
- improve weg items editor experience by adding live reload feature.

### refactor
- deprecate `onClick` and add new `onClickV2` on toolbar modules.

### fix
- bad translations keys.
- no restoring dock on closing fullscreened app.

## [1.8.12]
### fix
- app installed by msix no opening.

## [1.8.11]
### fix
- remove unnecessary 1px padding on toolbar.

## [1.8.10]
### enhancements
- remove unnecessary loop on taskbar hiding function.

### fix
- no loading translations correctly on update modal.

## [1.8.9]
### enhancements
- add translation to the rest of apps (dock, toolbar, and update modal).

### fix
- not hiding the taskbar at start.
- opening multiple instances of the app.

## [1.8.8]
### fix
- app not running on startup

## [1.8.7]
### fix
- no updating themes on changes saved.

## [1.8.6]
### features
- Add multi-language support! 🥳.
- Add default media input/output selectors to media module in fancy toolbar.
- Add start module to dock/taskbar (opens start menu).

### enhancements
- Flat default themes to allow easier overrides.

### fix
- Fix zorder on hovering on weg and toolbar respectively to wm borders.
- Applying bad themes on apps.
- Not hiding the taskbar at start.

## [1.8.5]
### fix
- no executing seelen after update installation

## [1.8.4]
## [1.8.3]
### refactor
- migrate settings files from `$USER/.config/seelen` to `$APPDATA/com.seelen.seelen-ui`
- load uwp apps info asynchronously

### fix
- crash on move toolbar item
- can not remove media module

## [1.8.2]
### features
- fancy toolbar items now can be dragged of position.
- using fancy toolbar's layout now can be modified and saved as custom.yml.

## [1.8.1]
### features
- styles can be specified in fancy toolbar placeholder item.
- fancy toolbar item now will have an unique id, this can be specified in the placeholder file.

### enhancements
- replace "bluetooth" for "devices" on bundled fancy toolbar placeholders.

## [1.8.0]
### features
- Media module added to the toolbar.
- Media module added to SeelenWeg.

  ![Media Module Example](documentation/images/media_module_preview.png)

- SeelenWeg now has a context menu (Right Click Menu).

### enhancements
- enhancements on fullscreen events.

### refactor
- remove Default Wave animation on seelenweg (users will be able to add their own animations).

### fix
- no updating colors correctly on change light or dark mode on windows settings.
- window manager enabled by default for new users.
- showing tray icons with empty name.
- no focusing seelen settings if it was minimized.

## [1.7.7]
### fix
- no registering system events (battery/network/etc)

## [1.7.6]
### enhancements
- improve logging on dev mode and fix missing target on production logged errors.
- improve fullscreen matching.

### fix
- network icon showing incorrect interface icon (lan instead wifi).
- no updating adapters list and using adapter on network changes.

## [1.7.5]
## [1.7.4]
### enhancements
- improvements on workflows to auto upload artifacts to the store.

## [1.7.3]
### enhancements
- improvements on fullscreen events.

## [1.7.2]
### enhancements
- disable tiling window manager on windows 10 from UI (can be forced on settings.json file)

### fix
- app crashing on windows 10
- empty tray icons list on windows 10

## [1.7.1]
### enhancements
- separate `information` and `developer tools` tabs in the settings.
- add a option to open the install path in explorer.
- focus settings window if already exist.
- better performance on canceling changes in settings.
- avoid loading innecesary files in modules that are not used.
- update pinned apps path by filename on open (some apps change of path on updates so this will fix that).
- show empty message on toolbar when no wlan networks are found.

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