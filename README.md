# Dryv

Dryv is a video decoder implemented entirely in Rust, free from third-party dependencies. Currently, it supports AVC with plans to incorporate HEVC support in the future.

## Features

- [x] **Atom decoding**
- [x] **CABAC decoding**
- [x] **Inverse quantization**
- [x] **Inverse transform**
- [x] **Intra frame prediction**
- [ ] **Inter frame prediction**
- [ ] **Frame cropping**
- [ ] **CAVLC decoding**
- [ ] **Deblocking filter**
- [ ] **Display matrix transformations**
- [ ] **HEVC support**

## Usage


```bash
dryv <video-path> [-d]
```

After running it you'll find the first frame from the video in `./temp/yuv_frame`.

### Options

  `<video-path>`: The path to the video file you want to decode.

### Additional Options

  `-d`: Include this flag to print information about the video, such as it's dimensions, codec, duration.
