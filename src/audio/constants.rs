use libc::uint64_t;
use ffi::*;

bitflags! {
    pub flags ChannelLayout: uint64_t {
        const CHANNEL_LAYOUT_NATIVE            = AV_CH_LAYOUT_NATIVE            as uint64_t,
        const CHANNEL_LAYOUT_MONO              = AV_CH_LAYOUT_MONO              as uint64_t,
        const CHANNEL_LAYOUT_STEREO            = AV_CH_LAYOUT_STEREO            as uint64_t,
        const CHANNEL_LAYOUT_2POINT1           = AV_CH_LAYOUT_2POINT1           as uint64_t,
        const CHANNEL_LAYOUT_2_1               = AV_CH_LAYOUT_2_1               as uint64_t,
        const CHANNEL_LAYOUT_SURROUND          = AV_CH_LAYOUT_SURROUND          as uint64_t,
        const CHANNEL_LAYOUT_3POINT1           = AV_CH_LAYOUT_3POINT1           as uint64_t,
        const CHANNEL_LAYOUT_4POINT0           = AV_CH_LAYOUT_4POINT0           as uint64_t,
        const CHANNEL_LAYOUT_4POINT1           = AV_CH_LAYOUT_4POINT1           as uint64_t,
        const CHANNEL_LAYOUT_2_2               = AV_CH_LAYOUT_2_2               as uint64_t,
        const CHANNEL_LAYOUT_QUAD              = AV_CH_LAYOUT_QUAD              as uint64_t,
        const CHANNEL_LAYOUT_5POINT0           = AV_CH_LAYOUT_5POINT0           as uint64_t,
        const CHANNEL_LAYOUT_5POINT1           = AV_CH_LAYOUT_5POINT1           as uint64_t,
        const CHANNEL_LAYOUT_5POINT0_BACK      = AV_CH_LAYOUT_5POINT0_BACK      as uint64_t,
        const CHANNEL_LAYOUT_5POINT1_BACK      = AV_CH_LAYOUT_5POINT1_BACK      as uint64_t,
        const CHANNEL_LAYOUT_6POINT0           = AV_CH_LAYOUT_6POINT0           as uint64_t,
        const CHANNEL_LAYOUT_6POINT0_FRONT     = AV_CH_LAYOUT_6POINT0_FRONT     as uint64_t,
        const CHANNEL_LAYOUT_HEXAGONAL         = AV_CH_LAYOUT_HEXAGONAL         as uint64_t,
        const CHANNEL_LAYOUT_6POINT1           = AV_CH_LAYOUT_6POINT1           as uint64_t,
        const CHANNEL_LAYOUT_6POINT1_BACK      = AV_CH_LAYOUT_6POINT1_BACK      as uint64_t,
        const CHANNEL_LAYOUT_6POINT1_FRONT     = AV_CH_LAYOUT_6POINT1_FRONT     as uint64_t,
        const CHANNEL_LAYOUT_7POINT0           = AV_CH_LAYOUT_7POINT0           as uint64_t,
        const CHANNEL_LAYOUT_7POINT0_FRONT     = AV_CH_LAYOUT_7POINT0_FRONT     as uint64_t,
        const CHANNEL_LAYOUT_7POINT1           = AV_CH_LAYOUT_7POINT1           as uint64_t,
        const CHANNEL_LAYOUT_7POINT1_WIDE      = AV_CH_LAYOUT_7POINT1_WIDE      as uint64_t,
        const CHANNEL_LAYOUT_7POINT1_WIDE_BACK = AV_CH_LAYOUT_7POINT1_WIDE_BACK as uint64_t,
        const CHANNEL_LAYOUT_OCTAGONAL         = AV_CH_LAYOUT_OCTAGONAL         as uint64_t,
        const CHANNEL_LAYOUT_HEXADECAGONAL     = AV_CH_LAYOUT_HEXADECAGONAL     as uint64_t,
        const CHANNEL_LAYOUT_STEREO_DOWNMIX    = AV_CH_LAYOUT_STEREO_DOWNMIX    as uint64_t,
    }
}
