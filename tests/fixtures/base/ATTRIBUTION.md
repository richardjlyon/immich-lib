# Base Image Attribution

All base images are sourced from Lorem Picsum (https://picsum.photos), which provides
free-to-use images from Unsplash under the Unsplash License.

## Images Used

32 unique base images, one per test scenario:

### Winner Selection (W1-W8)
- base_w1.jpg - base_w8.jpg

### Consolidation (C1-C8)
- base_c1.jpg - base_c8.jpg

### Conflict Detection (F1-F7)
- base_f1.jpg - base_f7.jpg

### Edge Cases (X1-X5, X7, X9-X11)
- base_x1.jpg, base_x2.jpg, base_x3.jpg, base_x4.jpg, base_x5.jpg
- base_x7.jpg (PNG test)
- base_x9.jpg, base_x10.jpg, base_x11.jpg (Unicode, old date, future date)

Note: X6 (HEIC) and X8 (RAW) scenarios removed - cannot generate valid files without proprietary encoders.

## License

All images are provided under the Unsplash License, which allows for free use in
commercial and non-commercial projects without attribution requirement (though
attribution is appreciated).

https://unsplash.com/license

## Generation

Images were downloaded with unique seed values to ensure each scenario has a
visually distinct base image, preventing CLIP from grouping unrelated scenarios
together during duplicate detection.

Source URLs: `https://picsum.photos/seed/{scenario}scenario/600/400`
