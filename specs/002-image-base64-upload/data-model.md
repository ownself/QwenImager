# Data Model: Image Base64 Upload

**Feature Branch**: `002-image-base64-upload`  
**Date**: 2026-02-27

## No Data Model Changes

This feature does not introduce any new entities, modify existing schemas, or change data storage patterns.

The modification is purely in the API request construction layer:
- **Before**: Local file paths are converted to `file://` URLs and sent as string values
- **After**: Local file paths are read as binary, Base64-encoded, and sent as `data:{mime};base64,{data}` strings

The same string fields (`image` in img2img, `image_url` in translation) carry the encoded data — no structural change is needed.
