#include <stdio.h>

#include <libavcodec/avcodec.h>

int main(void)
{
    const AVCodec* encoder = NULL;
    void* opaque = NULL;

    while (encoder = av_codec_iterate(&opaque)) {
        if (!av_codec_is_encoder(encoder)) {
            continue;
        }

        const char* type = "";
        switch (encoder->type) {
        case AVMEDIA_TYPE_VIDEO:
            type = "V";
            break;
        case AVMEDIA_TYPE_AUDIO:
            type = "A";
            break;
        case AVMEDIA_TYPE_SUBTITLE:
            type = "S";
            break;
        default:
            break;
        }

        if (strncmp(type, "", 1) == 0) {
            continue;
        }

        const char* codec = avcodec_get_name(encoder->id);
        printf("[%s] %s (codec: %s)\n", type, encoder->name, codec);
    }

    return 0;
}
