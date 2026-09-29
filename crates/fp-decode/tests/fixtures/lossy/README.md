# Lossy fixtures

`tone.ogg` (Vorbis) holds 22 000
stereo frames of a 1 kHz sine at 44.1 kHz, generated for this project with
GStreamer:

```sh
gst-launch-1.0 audiotestsrc num-buffers=22 samplesperbuffer=1000 freq=1000 volume=0.5 \
  ! audio/x-raw,rate=44100,channels=2 ! audioconvert ! vorbisenc quality=0.3 ! oggmux \
  ! filesink location=tone.ogg
```
