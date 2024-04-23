/// https://libtiff.gitlab.io/libtiff/specification/coverage.html

/// 254
pub struct NewSubfileType(u32);


/// 256
pub struct ImageWidth(u32);


/// 257
pub struct ImageHeight(u32);


/// 258
pub struct BitsPerSample(Vec<u16>);


/// 259
pub struct Compression(u16);


/// 262
pub struct PhotometricInterpretation(u16);


/// 270
pub struct ImageDescription(String);


/// 277
pub struct SamplesPerPixel(u16);

/// 284
pub struct PlanarConfiguration(u16);

/// 322
pub struct TileWidth(u32);

/// 323
pub struct TileHeight(u16);

/// 324
pub struct TileOffsets(Vec<u32>);

/// 325
pub struct TileByteCounts(Vec<u32>);

/// 32997
pub struct ImageDepth(u16);

/// 34675
pub struct ICCProfile(Vec<u8>);

