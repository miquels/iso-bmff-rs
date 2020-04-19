use iso_bmff_macros::def_box;

// File Type Box
def_box! {
    aligned(8) class FileTypeBox
       extends Box("ftyp") {
       unsigned int(32) major_brand;
       unsigned int(32) minor_version;
       unsigned int(32) compatible_brands[];                  // to end of the box
    }
}

// Media Data Box
def_box! {
    aligned(8) class MediaDataBox extends Box("mdat") {
       bit(8) data[];
    }
}

// Free Space Box
def_box! {
    aligned(8) class FreeSpaceBox extends Box(free_type) {
       unsigned int(8) data[];
    }
}

// Progressive Download Information Box
def_box! {
    aligned(8) class ProgressiveDownloadInfoBox
          extends FullBox("pdin", version = 0, 0) {
       for (i=0; ; i++) {   // to end of box
          unsigned int(32) rate;
          unsigned int(32) initial_delay;
       }
    }
}

// Movie Box
def_box! {
    aligned(8) class MovieBox extends Box("moov"){
    }
}

// Movie Header Box
def_box! {
    aligned(8) class MovieHeaderBox extends FullBox("mvhd", version, 0) {
       if (version==1) {
          unsigned int(64) creation_time;
          unsigned int(64) modification_time;
          unsigned int(32) timescale;
          unsigned int(64) duration;
       } else { // version==0
          unsigned int(32) creation_time;
          unsigned int(32) modification_time;
          unsigned int(32) timescale;
          unsigned int(32) duration;
       }
       template int(32) rate = 0x00010000; // typically 1.0
       template int(16) volume = 0x0100;    // typically, full volume
       const bit(16) reserved = 0;
       const unsigned int(32)[2] reserved = 0;
       template int(32)[9] matrix =
          { 0x00010000,0,0,0,0x00010000,0,0,0,0x40000000 };
          // Unity matrix
       bit(32)[6] pre_defined = 0;
       unsigned int(32) next_track_ID;
    }
}

// Track Box
def_box! {
    aligned(8) class TrackBox extends Box("trak") {
    }
}

// Track Header Box
def_box! {
    aligned(8) class TrackHeaderBox
       extends FullBox("tkhd", version, flags){
       if (version==1) {
          unsigned int(64) creation_time;
          unsigned int(64) modification_time;
          unsigned int(32) track_ID;
          const unsigned int(32) reserved = 0;
          unsigned int(64) duration;
       } else { // version==0
          unsigned int(32) creation_time;
          unsigned int(32) modification_time;
          unsigned int(32) track_ID;
          const unsigned int(32) reserved = 0;
          unsigned int(32) duration;
       }
       const unsigned int(32)[2] reserved = 0;
       template int(16) layer = 0;
       template int(16) alternate_group = 0;
       template int(16) volume = {if track_is_audio 0x0100 else 0};
       const unsigned int(16) reserved = 0;
       template int(32)[9] matrix=
          { 0x00010000,0,0,0,0x00010000,0,0,0,0x40000000 };
          // unity matrix
       unsigned int(32) width;
       unsigned int(32) height;
    }
}

def_box! {
    aligned(8) class TrackReferenceBox extends Box("tref") {
    }
}

def_box! {
    aligned(8) class TrackReferenceTypeBox (unsigned int(32) reference_type) extends
    Box(reference_type) {
       unsigned int(32) track_IDs[];
    }
}

def_box! {
    aligned(8) class TrackGroupBox extends Box("trgr") {
    }
}

def_box! {
    aligned(8) class TrackGroupTypeBox(unsigned int(32) track_group_type) extends
    FullBox(track_group_type, version = 0, flags = 0)
    {
       unsigned int(32) track_group_id;
       // the remaining data may be specified for a particular track_group_type
    }
}

// Media Box
def_box! {
    aligned(8) class MediaBox extends Box("mdia") {
    }
}

// Media Header Box
def_box! {
    aligned(8) class MediaHeaderBox extends FullBox("mdhd", version, 0) {
       if (version==1) {
          unsigned int(64) creation_time;
          unsigned int(64) modification_time;
          unsigned int(32) timescale;
          unsigned int(64) duration;
       } else { // version==0
          unsigned int(32) creation_time;
          unsigned int(32) modification_time;
          unsigned int(32) timescale;
          unsigned int(32) duration;
       }
       bit(1)   pad = 0;
       unsigned int(5)[3]   language;   // ISO-639-2/T language code
       unsigned int(16) pre_defined = 0;
    }
}

// Handler Reference Box
def_box! {
    aligned(8) class HandlerBox extends FullBox("hdlr", version = 0, 0) {
       unsigned int(32) pre_defined = 0;
       unsigned int(32) handler_type;
       const unsigned int(32)[3] reserved = 0;
       string   name;
    }
}

// Media Information Box
def_box! {
    aligned(8) class MediaInformationBox extends Box("minf") {
    }
}

// Null Media Header Box
def_box! {
    aligned(8) class NullMediaHeaderBox
       extends FullBox("nmhd", version = 0, flags) {
     }
}

// Extended language tag
def_box! {
    aligned(8) class ExtendedLanguageBox extends FullBox("elng", 0, 0) {
       string   extended_language;
    }
}

// Sample Table Box
def_box! {
    aligned(8) class SampleTableBox extends Box("stbl") {
    }
}

def_box! {
    aligned(8) abstract class SampleEntry (unsigned int(32) format)
       extends Box(format){
       const unsigned int(8)[6] reserved = 0;
       unsigned int(16) data_reference_index;
    }
}

def_box! {
    class BitRateBox extends Box("btrt"){
       unsigned int(32) bufferSizeDB;
       unsigned int(32) maxBitrate;
       unsigned int(32) avgBitrate;
    }
}

def_box! {
    aligned(8) class SampleDescriptionBox (unsigned int(32) handler_type)
       extends FullBox("stsd", version, 0){
       int i ;
       unsigned int(32) entry_count;
       for (i = 1 ; i <= entry_count ; i++){
          SampleEntry();    // an instance of a class derived from SampleEntry
       }
    }
}

// Degradation Priority Box
def_box! {
    aligned(8) class DegradationPriorityBox
       extends FullBox("stdp", version = 0, 0) {
       int i;
       for (i=0; i < sample_count; i++) {
          unsigned int(16) priority;
       }
    }
}

// Decoding Time to Sample Box
def_box! {
    aligned(8) class TimeToSampleBox
       extends FullBox("stts", version = 0, 0) {
       unsigned int(32) entry_count;
          int i;
       for (i=0; i < entry_count; i++) {
          unsigned int(32) sample_count;
          unsigned int(32) sample_delta;
       }
    }
}

// Composition Time to Sample Box
def_box! {
    aligned(8) class CompositionOffsetBox
       extends FullBox("ctts", version, 0) {
       unsigned int(32) entry_count;
          int i;
       if (version==0) {
          for (i=0; i < entry_count; i++) {
             unsigned int(32) sample_count;
             unsigned int(32) sample_offset;
          }
       }
       else if (version == 1) {
          for (i=0; i < entry_count; i++) {
             unsigned int(32) sample_count;
             signed   int(32) sample_offset;
          }
       }
    }
}

// Composition to Decode Box
def_box! {
    class CompositionToDecodeBox extends FullBox("cslg", version, 0) {
       if (version==0) {
          signed int(32) compositionToDTSShift;
          signed int(32) leastDecodeToDisplayDelta;
          signed int(32) greatestDecodeToDisplayDelta;
          signed int(32) compositionStartTime;
          signed int(32) compositionEndTime;
       } else {
          signed int(64) compositionToDTSShift;
          signed int(64) leastDecodeToDisplayDelta;
          signed int(64) greatestDecodeToDisplayDelta;
          signed int(64) compositionStartTime;
          signed int(64) compositionEndTime;
       }
    }
}

// Sync Sample Box
def_box! {
    aligned(8) class SyncSampleBox
       extends FullBox("stss", version = 0, 0) {
       unsigned int(32) entry_count;
       int i;
       for (i=0; i < entry_count; i++) {
          unsigned int(32) sample_number;
       }
     }
}

// Shadow Sync Sample Box
def_box! {
    aligned(8) class ShadowSyncSampleBox
       extends FullBox("stsh", version = 0, 0) {
       unsigned int(32) entry_count;
       int i;
       for (i=0; i < entry_count; i++) {
          unsigned int(32) shadowed_sample_number;
          unsigned int(32) sync_sample_number;
       }
     }
}

// Independent and Disposable Samples Box
def_box! {
    aligned(8) class SampleDependencyTypeBox
       extends FullBox("sdtp", version = 0, 0) {
       for (i=0; i < sample_count; i++){
          unsigned int(2) is_leading;
          unsigned int(2) sample_depends_on;
          unsigned int(2) sample_is_depended_on;
          unsigned int(2) sample_has_redundancy;
       }
    }
}

// Edit Box
def_box! {
    aligned(8) class EditBox extends Box("edts") {
    }
}

// Edit List Box
def_box! {
    aligned(8) class EditListBox extends FullBox("elst", version, 0) {
       unsigned int(32) entry_count;
       for (i=1; i <= entry_count; i++) {
          if (version==1) {
             unsigned int(64) segment_duration;
             int(64) media_time;
          } else { // version==0
             unsigned int(32) segment_duration;
             int(32) media_time;
          }
          int(16) media_rate_integer;
          int(16) media_rate_fraction = 0;
       }
    }
}

// Data Information Box
def_box! {
    aligned(8) class DataInformationBox extends Box("dinf") {
    }
}

def_box! {
    aligned(8) class DataEntryUrlBox (bit(24) flags)
       extends FullBox("url ", version = 0, flags) {
       string   location;
    }
}

def_box! {
    aligned(8) class DataEntryUrnBox (bit(24) flags)
       extends FullBox("urn ", version = 0, flags) {
       string   name;
       string   location;
    }
}

def_box! {
    aligned(8) class DataReferenceBox
       extends FullBox("dref", version = 0, 0) {
       unsigned int(32) entry_count;
       for (i=1; i <= entry_count; i++) {
          DataEntryBox(entry_version, entry_flags) data_entry;
       }
    }
}

// Sample To Chunk Box
def_box! {
    aligned(8) class SampleToChunkBox
       extends FullBox("stsc", version = 0, 0) {
       unsigned int(32) entry_count;
       for (i=1; i <= entry_count; i++) {
          unsigned int(32) first_chunk;
          unsigned int(32) samples_per_chunk;
          unsigned int(32) sample_description_index;
       }
    }
}

def_box! {
    aligned(8) class ChunkOffsetBox
       extends FullBox("stco", version = 0, 0) {
       unsigned int(32) entry_count;
       for (i=1; i <= entry_count; i++) {
          unsigned int(32) chunk_offset;
       }
    }
}

def_box! {
    aligned(8) class ChunkLargeOffsetBox
       extends FullBox("co64", version = 0, 0) {
       unsigned int(32) entry_count;
       for (i=1; i <= entry_count; i++) {
          unsigned int(64) chunk_offset;
       }
    }
}

// Padding Bits Box
def_box! {
    aligned(8) class PaddingBitsBox extends FullBox("padb", version = 0, 0) {
       unsigned int(32) sample_count;
       int i;
       for (i=0; i < ((sample_count + 1)/2); i++) {
          bit(1)   reserved = 0;
          bit(3)   pad1;
          bit(1)   reserved = 0;
          bit(3)   pad2;
       }
     }
}

// Sub-Sample Information Box
def_box! {
    aligned(8) class SubSampleInformationBox
       extends FullBox("subs", version, flags) {
       unsigned int(32) entry_count;
       int i,j;
       for (i=0; i < entry_count; i++) {
          unsigned int(32) sample_delta;
          unsigned int(16) subsample_count;
          if (subsample_count > 0) {
             for (j=0; j < subsample_count; j++) {
                if(version == 1)
                {
                   unsigned int(32) subsample_size;
                }
                else
                {
                   unsigned int(16) subsample_size;
                }
                unsigned int(8) subsample_priority;
                unsigned int(8) discardable;
                unsigned int(32) codec_specific_parameters;
             }
          }
       }
    }
}

// Sample Auxiliary Information Sizes Box
def_box! {
    aligned(8) class SampleAuxiliaryInformationSizesBox
       extends FullBox("saiz", version = 0, flags)
    {
       if (flags & 1) {
          unsigned int(32) aux_info_type;
          unsigned int(32) aux_info_type_parameter;
       }
       unsigned int(8) default_sample_info_size;
       unsigned int(32) sample_count;
       if (default_sample_info_size == 0) {
          unsigned int(8) sample_info_size[ sample_count ];
       }
    }
}

// Sample Auxiliary Information Offsets Box
def_box! {
    aligned(8) class SampleAuxiliaryInformationOffsetsBox
       extends FullBox("saio", version, flags)
    {
       if (flags & 1) {
          unsigned int(32) aux_info_type;
          unsigned int(32) aux_info_type_parameter;
       }
       unsigned int(32) entry_count;
       if ( version == 0 ) {
          unsigned int(32) offset[ entry_count ];
       }
       else {
          unsigned int(64) offset[ entry_count ];
       }
    }
}

// Movie Extends Box
def_box! {
    aligned(8) class MovieExtendsBox extends Box("mvex"){
    }
}

// Movie Extends Header Box
def_box! {
    aligned(8) class MovieExtendsHeaderBox extends FullBox("mehd", version, 0) {
       if (version==1) {
          unsigned int(64) fragment_duration;
       } else { // version==0
          unsigned int(32) fragment_duration;
       }
    }
}

// Track Extends Box
def_box! {
    aligned(8) class TrackExtendsBox extends FullBox("trex", 0, 0){
       unsigned int(32) track_ID;
       unsigned int(32) default_sample_description_index;
       unsigned int(32) default_sample_duration;
       unsigned int(32) default_sample_size;
       unsigned int(32) default_sample_flags;
    }
}

// Movie Fragment Box
def_box! {
    aligned(8) class MovieFragmentBox extends Box("moof"){
    }
}

// Movie Fragment Header Box
def_box! {
    aligned(8) class MovieFragmentHeaderBox
             extends FullBox("mfhd", 0, 0){
       unsigned int(32) sequence_number;
    }
}

// Track Fragment Box
def_box! {
    aligned(8) class TrackFragmentBox extends Box("traf"){
    }
}

// Track Fragment Header Box
def_box! {
    aligned(8) class TrackFragmentHeaderBox
             extends FullBox("tfhd", 0, tf_flags){
       unsigned int(32) track_ID;
       // all the following are optional fields
       unsigned int(64) base_data_offset;
       unsigned int(32) sample_description_index;
       unsigned int(32) default_sample_duration;
       unsigned int(32) default_sample_size;
       unsigned int(32) default_sample_flags
    }
}

// Track Fragment Run Box
def_box! {
    aligned(8) class TrackRunBox
             extends FullBox("trun", version, tr_flags) {
       unsigned int(32) sample_count;
       // the following are optional fields
       signed int(32) data_offset;
       unsigned int(32) first_sample_flags;
       // all fields in the following array are optional
       {
          unsigned int(32) sample_duration;
          unsigned int(32) sample_size;
          unsigned int(32) sample_flags
          if (version == 0)
             { unsigned int(32)   sample_composition_time_offset; }
          else
             { signed int(32)     sample_composition_time_offset; }
       }[ sample_count ]
    }
}

// Movie Fragment Random Access Box
def_box! {
    aligned(8) class MovieFragmentRandomAccessBox
     extends Box("mfra")
    {
    }
}

def_box! {
    aligned(8) class TrackFragmentRandomAccessBox
     extends FullBox("tfra", version, 0) {
       unsigned int(32) track_ID;
       const unsigned int(26) reserved = 0;
       unsigned int(2)   length_size_of_traf_num;
       unsigned int(2)   length_size_of_trun_num;
       unsigned int(2)   length_size_of_sample_num;
       unsigned int(32) number_of_entry;
       for(i=1; i <= number_of_entry; i++){
          if(version==1){
             unsigned int(64) time;
             unsigned int(64) moof_offset;
          }else{
             unsigned int(32) time;
             unsigned int(32) moof_offset;
          }
          unsigned int((length_size_of_traf_num+1) * 8) traf_number;
          unsigned int((length_size_of_trun_num+1) * 8) trun_number;
          unsigned int((length_size_of_sample_num+1) * 8) sample_number;
       }
    }
}

// Semantics
def_box! {
    aligned(8) class MovieFragmentRandomAccessOffsetBox
     extends FullBox("mfro", version, 0) {
       unsigned int(32) size;
    }
}

// Semantics
def_box! {
    aligned(8) class TrackFragmentBaseMediaDecodeTimeBox
       extends FullBox("tfdt", version, 0) {
       if (version==1) {
          unsigned int(64) baseMediaDecodeTime;
       } else { // version==0
          unsigned int(32) baseMediaDecodeTime;
       }
    }
}

// Semantics
def_box! {
    aligned(8) class LevelAssignmentBox extends FullBox("leva", 0, 0)
    {
       unsigned int(8)   level_count;
       for (j=1; j <= level_count; j++) {
          unsigned int(32) track_id;
          unsigned int(1)   padding_flag;
          unsigned int(7)   assignment_type;
          if (assignment_type == 0) {
             unsigned int(32) grouping_type;
          }
          else if (assignment_type == 1) {
             unsigned int(32) grouping_type;
             unsigned int(32) grouping_type_parameter;
          }
          else if (assignment_type == 2) {} // no further syntax elements needed
          else if (assignment_type == 3) {} // no further syntax elements needed
          else if (assignment_type == 4) {
             unsigned int(32) sub_track_id;
          }
          // other assignment_type values are reserved
       }
    }
}

// Semantics
def_box! {
    class TrackExtensionPropertiesBox extends FullBox("trep", 0, 0) {
       unsigned int(32) track_id;
       // Any number of boxes may follow
    }
}

// Semantics
def_box! {
    class AlternativeStartupSequencePropertiesBox extends FullBox("assp", version, 0)
    {
       if (version == 0) {
          signed int(32)    min_initial_alt_startup_offset;
       }
       else if (version == 1) {
          unsigned int(32) num_entries;
          for (j=1; j <= num_entries; j++) {
             unsigned int(32) grouping_type_parameter;
             signed int(32)    min_initial_alt_startup_offset;
          }
       }
    }
}

// Sample to Group Box
def_box! {
    aligned(8) class SampleToGroupBox
       extends FullBox("sbgp", version, 0)
    {
       unsigned int(32) grouping_type;
       if (version == 1) {
          unsigned int(32) grouping_type_parameter;
       }
       unsigned int(32) entry_count;
       for (i=1; i <= entry_count; i++)
       {
          unsigned int(32) sample_count;
          unsigned int(32) group_description_index;
       }
    }
}

def_box! {
    abstract class SampleGroupDescriptionEntry (unsigned int(32) grouping_type)
    {
    }
}

def_box! {
    abstract class VisualSampleGroupEntry (unsigned int(32) grouping_type) extends
    SampleGroupDescriptionEntry (grouping_type)
    {
    }
}

def_box! {
    abstract class AudioSampleGroupEntry (unsigned int(32) grouping_type) extends
    SampleGroupDescriptionEntry (grouping_type)
    {
    }
}

def_box! {
    abstract class HintSampleGroupEntry (unsigned int(32) grouping_type) extends
    SampleGroupDescriptionEntry (grouping_type)
    {
    }
}

def_box! {
    abstract class SubtitleSampleGroupEntry (unsigned int(32) grouping_type) extends
    SampleGroupDescriptionEntry (grouping_type)
    {
    }
}

def_box! {
    abstract class TextSampleGroupEntry (unsigned int(32) grouping_type) extends
    SampleGroupDescriptionEntry (grouping_type)
    {
    }
}

def_box! {
    aligned(8) class SampleGroupDescriptionBox (unsigned int(32) handler_type)
       extends FullBox("sgpd", version, 0){
       unsigned int(32) grouping_type;
       if (version==1) { unsigned int(32) default_length; }
         if (version>=2) {
            unsigned int(32) default_sample_description_index;
         }
         unsigned int(32) entry_count;
         int i;
         for (i = 1 ; i <= entry_count ; i++){
            if (version==1) {
               if (default_length==0) {
                  unsigned int(32) description_length;
               }
            }
            SampleGroupEntry (grouping_type);
               // an instance of a class derived from SampleGroupEntry
               // that is appropriate and permitted for the media type
         }
    }
}

// User Data Box
def_box! {
    aligned(8) class UserDataBox extends Box("udta") {
    }
}

// Copyright Box
def_box! {
    aligned(8) class CopyrightBox
       extends FullBox("cprt", version = 0, 0) {
       const bit(1)   pad = 0;
       unsigned int(5)[3]   language;   // ISO-639-2/T language code
       string   notice;
    }
}

def_box! {
    aligned(8) class TrackSelectionBox
       extends FullBox("tsel", version = 0, 0) {
       template int(32) switch_group = 0;
       unsigned int(32) attribute_list[];      // to end of the box
    }
}

// Track kind
def_box! {
    aligned(8) class KindBox
       extends FullBox("kind", version = 0, 0) {
       string   schemeURI;
       string   value;
    }
}

// The Meta box
def_box! {
    aligned(8) class MetaBox (handler_type)
       extends FullBox("meta", version = 0, 0) {
       HandlerBox(handler_type)   theHandler;
       PrimaryItemBox       primary_resource;                    //   optional
       DataInformationBox   file_locations;                      //   optional
       ItemLocationBox      item_locations;                      //   optional
       ItemProtectionBox    protections;                         //   optional
       ItemInfoBox          item_infos;                          //   optional
       IPMPControlBox       IPMP_control;                        //   optional
       ItemReferenceBox     item_refs;                           //   optional
       ItemDataBox          item_data;                           //   optional
       Box   other_boxes[];                                      //   optional
    }
}

def_box! {
    aligned(8) class XMLBox
          extends FullBox("xml ", version = 0, 0) {
       string xml;
    }
}

def_box! {
    aligned(8) class BinaryXMLBox
          extends FullBox("bxml", version = 0, 0) {
       unsigned int(8) data[];    // to end of box
    }
}

// The Item Location Box
def_box! {
    aligned(8) class ItemLocationBox extends FullBox("iloc", version, 0) {
       unsigned int(4)   offset_size;
       unsigned int(4)   length_size;
       unsigned int(4)   base_offset_size;
       if ((version == 1) || (version == 2)) {
          unsigned int(4)   index_size;
       } else {
          unsigned int(4)   reserved;
       }
       if (version < 2) {
          unsigned int(16) item_count;
       } else if (version == 2) {
          unsigned int(32) item_count;
       }
       for (i=0; i<item_count; i++) {
          if (version < 2) {
             unsigned int(16) item_ID;
          } else if (version == 2) {
             unsigned int(32) item_ID;
          }
          if ((version == 1) || (version == 2)) {
             unsigned int(12) reserved = 0;
             unsigned int(4)   construction_method;
          }
          unsigned int(16) data_reference_index;
          unsigned int(base_offset_size*8) base_offset;
          unsigned int(16)     extent_count;
          for (j=0; j<extent_count; j++) {
             if (((version == 1) || (version == 2)) && (index_size > 0)) {
                unsigned int(index_size*8) extent_index;
             }
             unsigned int(offset_size*8) extent_offset;
             unsigned int(length_size*8) extent_length;
          }
       }
    }
}

// Primary Item Box
def_box! {
    aligned(8) class PrimaryItemBox
          extends FullBox("pitm", version, 0) {
       if (version == 0) {
          unsigned int(16) item_ID;
       } else {
          unsigned int(32) item_ID;
       }
    }
}

// Item Protection Box
def_box! {
    aligned(8) class ItemProtectionBox
          extends FullBox("ipro", version = 0, 0) {
       unsigned int(16) protection_count;
       for (i=1; i<=protection_count; i++) {
          ProtectionSchemeInfoBox protection_information;
       }
    }
}

def_box! {
    aligned(8) class ItemInfoExtension(unsigned int(32) extension_type)
    {
    }
}

def_box! {
    aligned(8) class FDItemInfoExtension() extends ItemInfoExtension ("fdel")
    {
       string            content_location;
       string            content_MD5;
       unsigned int(64) content_length;
       unsigned int(64) transfer_length;
       unsigned int(8)   entry_count;
       for (i=1; i <= entry_count; i++)
          unsigned int(32) group_id;
    }
}

def_box! {
    aligned(8) class ItemInfoEntry
          extends FullBox("infe", version, 0) {
       if ((version == 0) || (version == 1)) {
          unsigned int(16) item_ID;
          unsigned int(16) item_protection_index
          string            item_name;
          string            content_type;
          string            content_encoding; //optional
       }
       if (version == 1) {
          unsigned int(32) extension_type;      //optional
          ItemInfoExtension(extension_type);    //optional
       }
       if (version >= 2) {
          if (version == 2) {
             unsigned int(16) item_ID;
          } else if (version == 3) {
             unsigned int(32) item_ID;
          }
          unsigned int(16) item_protection_index;
          unsigned int(32) item_type;
             string            item_name;
             if (item_type=="mime") {
                string         content_type;
                string         content_encoding;        //optional
             } else if (item_type == "uri ") {
                string         item_uri_type;
             }
         }
    }
}

def_box! {
    aligned(8) class ItemInfoBox
          extends FullBox("iinf", version, 0) {
       if (version == 0) {
          unsigned int(16) entry_count;
       } else {
          unsigned int(32) entry_count;
       }
       ItemInfoEntry[ entry_count ]     item_infos;
    }
}

// Additional Metadata Container Box
def_box! {
    aligned(8) class AdditionalMetadataContainerBox extends Box("meco") {
    }
}

// Metabox Relation Box
def_box! {
    aligned(8) class MetaboxRelationBox
       extends FullBox("mere", version=0, 0) {
       unsigned int(32) first_metabox_handler_type;
       unsigned int(32) second_metabox_handler_type;
       unsigned int(8)   metabox_relation;
    }
}

// MPEG-7 metadata
def_box! {
    aligned(8) class ItemDataBox extends Box("idat") {
       bit(8) data[];
    }
}

// Protection Scheme Information Box
def_box! {
    aligned(8) class ProtectionSchemeInfoBox(fmt) extends Box("sinf") {
       OriginalFormatBox(fmt) original_format;
         SchemeTypeBox                 scheme_type_box;    // optional
         SchemeInformationBox          info;        // optional
    }
}

// Original Format Box
def_box! {
    aligned(8) class OriginalFormatBox(codingname) extends Box ("frma") {
       unsigned int(32) data_format = codingname;
                         // format of decrypted, encoded data (in case of protection)
                         // or un-transformed sample entry (in case of restriction
                         // and complete track information)
    }
}

// Scheme Type Box
def_box! {
    aligned(8) class SchemeTypeBox extends FullBox("schm", 0, flags) {
       unsigned int(32) scheme_type;     // 4CC identifying the scheme
       unsigned int(32) scheme_version; // scheme version
       if (flags & 0x000001) {
          unsigned int(8)   scheme_uri[];       // browser uri
       }
    }
}

// Scheme Information Box
def_box! {
    aligned(8) class SchemeInformationBox extends Box("schi") {
       Box   scheme_specific_data[];
    }
}

def_box! {
    aligned(8) class PartitionEntry extends Box("paen") {
       FilePartitionBox blocks_and_symbols;
       FECReservoirBox   FEC_symbol_locations; //optional
       FileReservoirBox File_symbol_locations; //optional
    }
}

def_box! {
    aligned(8) class FDItemInformationBox
          extends FullBox("fiin", version = 0, 0) {
       unsigned int(16) entry_count;
       PartitionEntry    partition_entries[ entry_count ];
       FDSessionGroupBox session_info;     //optional
       GroupIdToNameBox group_id_to_name; //optional
    }
}

// File Partition Box
def_box! {
    aligned(8) class FilePartitionBox
          extends FullBox("fpar", version, 0) {
       if (version == 0) {
          unsigned int(16) item_ID;
       } else {
          unsigned int(32) item_ID;
       }
       unsigned int(16) packet_payload_size;
       unsigned int(8)   reserved = 0;
       unsigned int(8)   FEC_encoding_ID;
       unsigned int(16) FEC_instance_ID;
       unsigned int(16) max_source_block_length;
       unsigned int(16) encoding_symbol_length;
       unsigned int(16) max_number_of_encoding_symbols;
       string            scheme_specific_info;
       if (version == 0) {
          unsigned int(16) entry_count;
       } else {
          unsigned int(32) entry_count;
       }
       for (i=1; i <= entry_count; i++) {
          unsigned int(16) block_count;
          unsigned int(32) block_size;
       }
    }
}

// FEC Reservoir Box
def_box! {
    aligned(8) class FECReservoirBox
          extends FullBox("fecr", version, 0) {
       if (version == 0) {
          unsigned int(16) entry_count;
       } else {
          unsigned int(32) entry_count;
       }
       for (i=1; i <= entry_count; i++) {
          if (version == 0) {
             unsigned int(16) item_ID;
          } else {
             unsigned int(32) item_ID;
          }
          unsigned int(32) symbol_count;
       }
    }
}

// FD Session Group Box
def_box! {
    aligned(8) class FDSessionGroupBox extends Box("segr") {
       unsigned int(16) num_session_groups;
       for(i=0; i < num_session_groups; i++) {
          unsigned int(8)   entry_count;
          for (j=0; j < entry_count; j++) {
             unsigned int(32) group_ID;
          }
          unsigned int(16) num_channels_in_session_group;
          for(k=0; k < num_channels_in_session_group; k++) {
             unsigned int(32) hint_track_id;
          }
       }
    }
}

// Group ID to Name Box
def_box! {
    aligned(8) class GroupIdToNameBox
          extends FullBox("gitn", version = 0, 0) {
       unsigned int(16) entry_count;
       for (i=1; i <= entry_count; i++) {
          unsigned int(32) group_ID;
          string            group_name;
       }
    }
}

// File Reservoir Box
def_box! {
    aligned(8) class FileReservoirBox
          extends FullBox("fire", version, 0) {
       if (version == 0) {
          unsigned int(16) entry_count;
       } else {
          unsigned int(32) entry_count;
       }
       for (i=1; i <= entry_count; i++) {
          if (version == 0) {
             unsigned int(16) item_ID;
          } else {
             unsigned int(32) item_ID;
          }
          unsigned int(32) symbol_count;
       }
    }
}

// Sub Track box
def_box! {
    aligned(8) class SubTrack extends Box("strk") {
    }
}

// Sub Track Information box
def_box! {
    aligned(8) class SubTrackInformation
       extends FullBox("stri", version = 0, 0){
       template int(16) switch_group = 0;
       template int(16) alternate_group = 0;
       template unsigned int(32) sub_track_ID = 0;
       unsigned int(32) attribute_list[]; // to the end of the box
    }
}

// Sub Track Definition box
def_box! {
    aligned(8) class SubTrackDefinition extends Box("strd") {
    }
}

// Sub Track Sample Group box
def_box! {
    aligned(8) class SubTrackSampleGroupBox
       extends FullBox("stsg", 0, 0){
       unsigned int(32) grouping_type;
       unsigned int(16) item_count;
       for(i = 0; i< item_count; i++)
          unsigned int(32) group_description_index;
    }
}

// Restricted Scheme Information box
def_box! {
    aligned(8) class RestrictedSchemeInfoBox(fmt) extends Box("rinf") {
       OriginalFormatBox(fmt) original_format;
       SchemeTypeBox           scheme_type_box;
       SchemeInformationBox    info;        // optional
    }
}

// Stereo video box
def_box! {
    aligned(8) class StereoVideoBox extends extends FullBox("stvi", version = 0, 0)
    {
       template unsigned int(30) reserved = 0;
       unsigned int(2)   single_view_allowed;
       unsigned int(32) stereo_scheme;
       unsigned int(32) length;
       unsigned int(8)[length] stereo_indication_type;
       Box[] any_box; // optional
    }
}

// Segment Index Box
def_box! {
    aligned(8) class SegmentIndexBox extends FullBox("sidx", version, 0) {
       unsigned int(32) reference_ID;
       unsigned int(32) timescale;
       if (version==0) {
             unsigned int(32) earliest_presentation_time;
             unsigned int(32) first_offset;
          }
          else {
             unsigned int(64) earliest_presentation_time;
             unsigned int(64) first_offset;
          }
       unsigned int(16) reserved = 0;
       unsigned int(16) reference_count;
       for(i=1; i <= reference_count; i++)
       {
          bit (1)           reference_type;
          unsigned int(31) referenced_size;
          unsigned int(32) subsegment_duration;
          bit(1)            starts_with_SAP;
          unsigned int(3)   SAP_type;
          unsigned int(28) SAP_delta_time;
       }
    }
}

// Subsegment Index Box
def_box! {
    aligned(8) class SubsegmentIndexBox extends FullBox("ssix", 0, 0) {
       unsigned int(32) subsegment_count;
       for( i=1; i <= subsegment_count; i++)
       {
          unsigned int(32) range_count;
          for ( j=1; j <= range_count; j++) {
             unsigned int(8) level;
             unsigned int(24) range_size;
          }
       }
    }
}

// Producer Reference Time Box
def_box! {
    aligned(8) class ProducerReferenceTimeBox extends FullBox("prft", version, 0) {
       unsigned int(32) reference_track_ID;
       unsigned int(64) ntp_timestamp;
       if (version==0) {
          unsigned int(32) media_time;
       } else {
          unsigned int(64) media_time;
       }
    }
}

// Complete Track Information Box
def_box! {
    aligned(8) class CompleteTrackInfoBox(fmt) extends Box("cinf") {
       OriginalFormatBox(fmt) original_format;
    }
}

// Sample Description Format
def_box! {
    class FDHintSampleEntry() extends SampleEntry ("fdp ") {
       unsigned int(16) hinttrackversion = 1;
       unsigned int(16) highestcompatibleversion = 1;
       unsigned int(16) partition_entry_ID;
       unsigned int(16) FEC_overhead;
       Box               additionaldata[];   //optional
    }
}

// FEC Information Box
def_box! {
    aligned(8) class FECInformationBox extends Box("feci") {
       unsigned int(8)   FEC_encoding_ID;
       unsigned int(16) FEC_instance_ID;
       unsigned int(16) source_block_number;
       unsigned int(16) encoding_symbol_ID;
    }
}

def_box! {
    class MPEG2TSReceptionSampleEntry extends MPEG2TSSampleEntry("rm2t") {
    }
}

def_box! {
    class MPEG2TSServerSampleEntry extends MPEG2TSSampleEntry("sm2t") {
    }
}

def_box! {
    class MPEG2TSSampleEntry(unsigned int(32) name) extends HintSampleEntry(name) {
       uint(16) hinttrackversion = 1;
       uint(16) highestcompatibleversion = 1;
       uint(8) precedingbyteslen;
       uint(8) trailingbyteslen;
       uint(1) precomputed_only_flag;
       uint(7) reserved;
       box      additionaldata[];
    }
}

def_box! {
    class ProtectedMPEG2TransportStreamSampleEntry
       extends MPEG2TransportStreamSampleEntry("pm2t") {
       ProtectionSchemeInfoBox    SchemeInformation;
    }
}

def_box! {
    aligned(8) class receivedRTCPpacket {
       unsigned int(8)      data[];
    }
}

def_box! {
    aligned(8) class receivedRTCPsample {
       unsigned int(16) packetcount;
       unsigned int(16) reserved;
       receivedRTCPpacket   packets[packetcount];
    }
}

def_box! {
    class ProtectedRtpReceptionHintSampleEntry
       extends RtpReceptionHintSampleEntry ("prtp") {
       ProtectionSchemeInfoBox    SchemeInformation;
    }
}

def_box! {
    class VisualRollRecoveryEntry() extends VisualSampleGroupEntry ("roll")
    {
       signed int(16) roll_distance;
    }
}

def_box! {
    class AudioRollRecoveryEntry() extends AudioSampleGroupEntry ("roll")
    {
       signed int(16) roll_distance;
    }
}

def_box! {
    class AudioPreRollEntry() extends AudioSampleGroupEntry ("prol")
    {
       signed int(16) roll_distance;
    }
}

// Rate Share Sample Group Entry
def_box! {
    class RateShareEntry() extends SampleGroupDescriptionEntry("rash") {
       unsigned int(16) operation_point_count;
       if (operation_point_count == 1) {
          unsigned int(16)     target_rate_share;
       }
       else {
          for (i=0; i < operation_point_count; i++) {
             unsigned int(32) available_bitrate;
             unsigned int(16) target_rate_share;
          }
       }
       unsigned int(32) maximum_bitrate;
       unsigned int(32) minimum_bitrate;
       unsigned int(8)   discard_priority;
    }
}

// Alternative Startup Sequences
def_box! {
    class AlternativeStartupEntry() extends VisualSampleGroupEntry ("alst")
    {
      unsigned int(16) roll_count;
      unsigned int(16) first_output_sample;
      for (i=1; i <= roll_count; i++)
        unsigned int(32) sample_offset[i];
      j=1;
      do { // optional, until the end of the structure
        unsigned int(16) num_output_samples[j];
        unsigned int(16) num_total_samples[j];
        j++;
      }
    }
}

// Random Access Point (RAP) Sample Grouping
def_box! {
    class VisualRandomAccessEntry() extends VisualSampleGroupEntry ("rap ")
    {
       unsigned int(1) num_leading_samples_known;
       unsigned int(7) num_leading_samples;
    }
}

// Temporal level sample grouping
def_box! {
    class TemporalLevelEntry() extends VisualSampleGroupEntry("tele")
    {
       bit(1)   level_independently_decodable;
       bit(7)   reserved=0;
    }
}

// Stream access point sample group
def_box! {
    class SAPEntry() extends SampleGroupDescriptionEntry("sap ")
    {
       unsigned int(1) dependent_flag;
       unsigned int(3) reserved;
       unsigned int(4) SAP_type;
    }
}

// Video media header
def_box! {
    aligned(8) class VideoMediaHeaderBox
       extends FullBox("vmhd", version = 0, 1) {
       template unsigned int(16) graphicsmode = 0;   // copy, see below
       template unsigned int(16)[3] opcolor = {0, 0, 0};
    }
}

// Sample entry
def_box! {
    class VisualSampleEntry(unsigned int(32) codingname) extends SampleEntry (codingname){
       unsigned int(16) pre_defined = 0;
       const unsigned int(16) reserved = 0;
       unsigned int(32)[3] pre_defined = 0;
       unsigned int(16) width;
       unsigned int(16) height;
       template unsigned int(32) horizresolution = 0x00480000; // 72 dpi
       template unsigned int(32) vertresolution = 0x00480000; // 72 dpi
       const unsigned int(32) reserved = 0;
       template unsigned int(16) frame_count = 1;
       string[32] compressorname;
       template unsigned int(16) depth = 0x0018;
       int(16) pre_defined = -1;
       // other boxes from derived specifications
       CleanApertureBox     clap;    // optional
       PixelAspectRatioBox pasp;     // optional
    }
}

def_box! {
    class PixelAspectRatioBox extends Box("pasp"){
       unsigned int(32) hSpacing;
       unsigned int(32) vSpacing;
    }
}

def_box! {
    class CleanApertureBox extends Box("clap"){
       unsigned int(32) cleanApertureWidthN;
       unsigned int(32) cleanApertureWidthD;
        unsigned int(32) cleanApertureHeightN;
        unsigned int(32) cleanApertureHeightD;
        unsigned int(32) horizOffN;
        unsigned int(32) horizOffD;
        unsigned int(32) vertOffN;
        unsigned int(32) vertOffD;
    }
}

// Colour information
def_box! {
    class ColourInformationBox extends Box("colr"){
       unsigned int(32) colour_type;
       if (colour_type == "nclx") /* on-screen colours */
       {
          unsigned int(16) colour_primaries;
          unsigned int(16) transfer_characteristics;
          unsigned int(16) matrix_coefficients;
          unsigned int(1) full_range_flag;
          unsigned int(7) reserved = 0;
       }
       else if (colour_type == "rICC")
       {
          ICC_profile;   // restricted ICC profile
       }
       else if (colour_type == "prof")
       {
          ICC_profile;   // unrestricted ICC profile
       }
    }
}

// Sound media header
def_box! {
    aligned(8) class SoundMediaHeaderBox
       extends FullBox("smhd", version = 0, 0) {
       template int(16) balance = 0;
       const unsigned int(16) reserved = 0;
    }
}

def_box! {
    class AudioSampleEntry(unsigned int(32) codingname) extends SampleEntry (codingname){
       const unsigned int(32)[2] reserved = 0;
       template unsigned int(16) channelcount = 2;
       template unsigned int(16) samplesize = 16;
       unsigned int(16) pre_defined = 0;
       const unsigned int(16) reserved = 0 ;
       template unsigned int(32) samplerate = { default samplerate of media}<<16;
       ChannelLayout();
       // we permit any number of DownMix or DRC boxes:
       DownMixInstructions() [];
       DRCCoefficientsBasic() [];
       DRCInstructionsBasic() [];
       DRCCoefficientsUniDRC() [];
       DRCInstructionsUniDRC() [];
       Box ();     // further boxes as needed
    }
}

def_box! {
    aligned(8) class SamplingRateBox extends FullBox("srat") {
       unsigned int(32) sampling_rate;
    }
}

def_box! {
    class AudioSampleEntryV1(unsigned int(32) codingname) extends SampleEntry (codingname){
       unsigned int(16) entry_version; // must be 1,
                                  // and must be in an stsd with version ==1
       const unsigned int(16)[3] reserved = 0;
       template unsigned int(16) channelcount; // must be correct
       template unsigned int(16) samplesize = 16;
       unsigned int(16) pre_defined = 0;
       const unsigned int(16) reserved = 0 ;
       template unsigned int(32) samplerate = 1<<16;
       // optional boxes follow
       SamplingRateBox();
       ChannelLayout();
       // we permit any number of DownMix or DRC boxes:
       DownMixInstructions() [];
       DRCCoefficientsBasic() [];
       DRCInstructionsBasic() [];
       DRCCoefficientsUniDRC() [];
       DRCInstructionsUniDRC() [];
       Box ();     // further boxes as needed
    }
}

// Channel layout
def_box! {
    aligned(8) class ChannelLayout extends FullBox("chnl") {
       unsigned int(8)   stream_structure;
       if (stream_structure & channelStructured) {    // 1
          unsigned int(8) definedLayout;
          if (definedLayout==0) {
             for (i = 1 ; i <= channelCount ; i++) {
                // channelCount comes from the sample entry
                unsigned int(8) speaker_position;
                if (speaker_position == 126) { // explicit position
                   signed int (16) azimuth;
                   signed int (8) elevation;
                }
             }
          } else {
             unsigned int(64) omittedChannelsMap;
                   // a "1" bit indicates "not in this track"
          }
       }
       if (stream_structure & objectStructured) { // 2
          unsigned int(8) object_count;
       }
    }
}

// Downmix Instructions
def_box! {
    aligned(8) class DownMixInstructions extends FullBox("dmix") {
       unsigned int(8) targetLayout;
       unsigned int(1) reserved = 0;
       unsigned int(7) targetChannelCount;
       bit(1) in_stream;
       unsigned int(7) downmix_ID;
       if (in_stream==0)
       { // downmix coefficients are out of stream and supplied here
          int i, j;
          for (i = 1 ; i <= targetChannelCount; i++){
             for (j=1; j <= baseChannelCount; j++) {
                bit(4) bs_downmix_coefficient;
             }
          }
       }
    }
}

def_box! {
    aligned(8) class LoudnessBaseBox extends FullBox(loudnessType) {
       unsigned int(3) reserved = 0;
       unsigned int(7) downmix_ID;      // matching downmix
       unsigned int(6) DRC_set_ID;      // to match a DRC box
       signed int(12) bs_sample_peak_level;
       signed int(12) bs_true_peak_level;
       unsigned int(4) measurement_system_for_TP;
       unsigned int(4) reliability_for_TP;
       unsigned int(8) measurement_count;
       int i;
       for (i = 1 ; i <= measurement_count; i++){
          unsigned int(8) method_definition;
          unsigned int(8) method_value;
          unsigned int(4) measurement_system;
          unsigned int(4) reliability;
       }
    }
}

def_box! {
    aligned(8) class TrackLoudnessInfo extends LoudnessBaseBox("tlou") { }
    aligned(8) class AlbumLoudnessInfo extends LoudnessBaseBox ("alou") { }
    aligned(8) class LoudnessBox extends Box("ludt") {
       loudness       TrackLoudnessInfo[]; // a set of one or more loudness boxes
       albumLoudness AlbumLoudnessInfo[]; // if applicable
    }
}

def_box! {
    class MetaDataSampleEntry(unsigned int(32) codingname) extends SampleEntry (codingname) {
       Box[] other_boxes; // optional
    }
}

def_box! {
    class XMLMetaDataSampleEntry() extends MetaDataSampleEntry ("metx") {
       string   content_encoding; // optional
       string   namespace;
       string   schema_location; // optional
       BitRateBox (); // optional
    }
}

def_box! {
    class TextConfigBox() extends Fullbox ("txtC", 0, 0) {
       string   text_config;
    }
}

def_box! {
    class TextMetaDataSampleEntry() extends MetaDataSampleEntry ("mett") {
       string   content_encoding; // optional
       string   mime_format;
       BitRateBox ();    // optional
       TextConfigBox (); // optional
    }
}

def_box! {
    aligned(8) class URIBox
          extends FullBox("uri ", version = 0, 0) {
       string   theURI;
    }
}

def_box! {
    aligned(8) class URIInitBox
          extends FullBox("uriI", version = 0, 0) {
       unsigned int(8) uri_initialization_data[];
    }
}

def_box! {
    class URIMetaSampleEntry() extends MetaDataSampleEntry ("urim") {
       URIbox         the_label;
       URIInitBox     init;    // optional
       BitRateBox ();          // optional
    }
}

// Sample entry
def_box! {
    class HintSampleEntry() extends SampleEntry (protocol) {
       unsigned int(8) data [];
    }
}

def_box! {
    class PlainTextSampleEntry(unsigned int(32) codingname) extends SampleEntry (codingname) {
    }
}

def_box! {
    class SimpleTextSampleEntry(unsigned int(32) codingname) extends PlainTextSampleEntry ("stxt") {
       string   content_encoding; // optional
       string   mime_format;
       BitRateBox ();             // optional
       TextConfigBox ();          // optional
    }
}

// Subtitle media header
def_box! {
    aligned(8) class SubtitleMediaHeaderBox
       extends FullBox ("sthd", version = 0, flags = 0){
    }
}

def_box! {
    class SubtitleSampleEntry(unsigned int(32) codingname) extends SampleEntry (codingname) {
    }
}

def_box! {
    class XMLSubtitleSampleEntry() extends SubtitleSampleEntry ("stpp") {
       string   namespace;
       string   schema_location; // optional
       string   auxiliary_mime_types;
                      // optional, required if auxiliary resources are present
       BitRateBox ();             // optional
    }
}

def_box! {
    class TextSubtitleSampleEntry() extends SubtitleSampleEntry ("sbtt") {
       string   content_encoding; // optional
       string   mime_format;
       BitRateBox ();             // optional
       TextConfigBox ();          // optional
    }
}

// Sample entry
def_box! {
    class FontSampleEntry(unsigned int(32) codingname) extends SampleEntry (codingname){
       //other boxes from derived specifications
       BitRateBox (); // optional
    }
}

