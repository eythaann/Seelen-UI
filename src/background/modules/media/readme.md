# Understanding this hell

- Media Device can be input device (microphone) or output (speaker)
- Media Device has sessions that are the apps using the device
  - sessions can have one or more streams.
  - sessions can implement transport protocol to indicate what is playing, we call this Media Player.
- Media Device has channels as example 2 channels (left and right), or 5.1, 7.1 etc.

now each device have master volume > volume per session > volume per stream > volume per channel