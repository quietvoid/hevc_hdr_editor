**`hevc_hdr_editor`** is a utility to losslessly edit HDR metadata in HEVC files.

&nbsp;

## **Building**
### **Toolchain**

The minimum Rust version to build **`hevc_hdr_editor`** is 1.79.0.

### **Release binary**
To build release binary in `target/release/hevc_hdr_editor` run:
```console
cargo build --release
```

&nbsp;

## Usage
```properties
hevc_hdr_editor [OPTIONS] --config <CONFIG> video.hevc
```

Supported input files:
- Raw HEVC bitstream
- Matroska (mkv) file with HEVC video track

### Options
* `--start-code` HEVC NALU start code to use when writing HEVC.
    - Options: `annex-b`, `four`
    - `annex-b` varies the start code, according to spec. Almost matches `x265` behaviour.
    - `four` is the default, writing a 4-byte start code all the time.

## Edit config

The config is expected to follow the template below:
```json5
{
    // Replace the SMPTE ST 2086 Mastering Display metadata
    "mdcv": {
        // Existing preset display primaries (BT.709, Display-P3 or BT.2020)
        // Options: "BT.709", "DisplayP3", "BT.2020"
        "preset": "DisplayP3",

        // If present, the specific primaries to use.
        // Example for x265 string:
        //   G(13250,34500)B(7500,3000)R(34000,16000)WP(15635,16450)L(10000000,1)
        "primaries": {
            // X, Y display primaries in RGB order as 16 bit integers
            "display_primaries_x": [34000, 13250, 7500],
            "display_primaries_y": [16000, 34500, 3000],
            "white_point": [15635, 16450]
        },

        // min, max mastering display luminance in nits
        "max_display_mastering_luminance": 1000,
        "min_display_mastering_luminance": 0.0001
    },

    // Replace the Content light level metadata
    "cll": {
        // MaxCLL value to set
        "max_content_light_level": 1000,
        // MaxFALL value to set
        "max_average_light_level": 400
    }
}
```
