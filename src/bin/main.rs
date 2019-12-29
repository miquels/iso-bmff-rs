use macrotest::def_box;

def_box! {
    aligned(8) class Box (unsigned int(32) boxtype = bla, optional unsigned int(8)[16] extended_type) {
        unsigned int(32) size;
        unsigned int(32) type = boxtype;
        if (size==1) {
            unsigned int(64) largesize;
        } else if (size==0) {
            unsigned int(16) loopy;
            // box extends to end of file
        } else if (size == "henk") {
            int(4242) henkie;
        }
        if (boxtype=="uuid") {
            unsigned int(8)[16] usertype = extended_type;
        }
    }
}

fn main() {
}

