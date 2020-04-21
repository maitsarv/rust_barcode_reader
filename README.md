# Rust barcode reader
Personal project for learning Rust language.

# About the project
* IS able to detect EAN-13 (and UPC-A) barcodes.
* Image processing part is very lightweight to make the detection fast.
* The image detection part might find false positives, but the control number check should filter them out.
* Does not go over every pixel, the row step is calculated based on the image height.

# Implementation
The current implementation reads in image files. 
There is a WASM implementation, that makes use of browsers MediaStream API for web-cam access: https://maitsarv.github.io/barcode-reader
