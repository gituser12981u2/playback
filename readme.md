**Flac Files**

    - STREAMINFO: this block contains details about the entire stream, like the sample rate, number of channels, total number of samples, and so on. This block is mandatory and there must be exactly one in every FLAC file.

    - PADDING: this block contains no data and is used for padding. It can be ignored for the purpose of reading metadata.

    - APPLICATION: this block contains data for third-party applications.

    - SEEKTABLE: this block contains seek points for seeking in the stream.

    - VORBIS_COMMENT: this block contains a list of comments, similar to ID3 tags in an MP3 file.

    - CUESHEET: this block is for storing cue sheets in the FLAC file.

    -PICTURE: this block is for storing pictures related to the FLAC file, like album covers.
