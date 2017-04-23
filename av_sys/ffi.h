#include <libavutil/avutil.h>
#include <libavutil/pixfmt.h>
#include <libavutil/imgutils.h>
#include <libavutil/timestamp.h>
#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
#include <libswscale/swscale.h>
#include <libswresample/swresample.h>

enum RUST_AV_CONSTANTS {
    RUST__int64_t__NOPTS_VALUE = AV_NOPTS_VALUE,
    RUST__c_int__AVERROR_EAGAIN = AVERROR(EAGAIN),
    RUST__c_int__AVERROR_EOF = AVERROR_EOF,
};
