#include <libavutil/avutil.h>
#include <libavutil/pixfmt.h>
#include <libavutil/imgutils.h>
#include <libavutil/timestamp.h>
#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
#include <libswscale/swscale.h>
#include <libswresample/swresample.h>

// Fix some problematic macros
/// <div rustbindgen replaces="AV_NOPTS_VALUE"></div>
const int64_t RUST_AV_NOPTS_VALUE = AV_NOPTS_VALUE;
/// <div rustbindgen replaces="AVERROR_EAGAIN"></div>
const int RUST_AVERROR_EAGAIN = AVERROR(EAGAIN);
/// <div rustbindgen replaces="AVERROR_EOF"></div>
const int RUST_AVERROR_EOF = AVERROR_EOF;
