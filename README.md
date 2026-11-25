# Rust Image to Ascii Renderer

Based on, and inpired by, [https://alexharri.com/blog/ascii-rendering](https://alexharri.com/blog/ascii-rendering)

Converts an image to ascii.

### shapes.py
It creates the shape defining vector for each character. You need a font file in the directory, put the path in `file_path`.\
It outputs `shapes.txt`, that is later read by the rust project.

### main.rs:
- `image`: put the path of your target image
- `ascii_width` `ascii_height` : dimensions of the resulting ascii 
  > ! Atention !  
  Depending on the values, the ascii art may be distorted. For example:  
  If the image is 1200x480, and you use dimensions of 40x40, it will be stretched


TODO: put an image here
