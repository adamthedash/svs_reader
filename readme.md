# SVS Reader

TIFF spec: https://docs.fileformat.com/image/tiff/  
https://www.itu.int/itudoc/itu-t/com16/tiff-fx/docs/tiff6.pdf    
https://libtiff.gitlab.io/libtiff/specification/coverage.html

BigTiff: https://www.awaresystems.be/imaging/tiff/bigtiff.html

Aperio SVS spec: https://openslide.org/formats/aperio/

jpeg2000: https://www.corsi.univr.it/documenti/OccorrenzaIns/matdid/matdid500139.pdf

# Requirements

1) List off the layer information
2) Read mpp/magnifcation
3) Read a region at a given location & layer
4) Read the thumbnail if it exists
5) Read a tile into a provided buffer in-place