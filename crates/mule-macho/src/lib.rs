use serde::Serialize;

#[derive(Serialize)]
pub struct Macho {
    pub header: Header,
    pub load_commands: Vec<LoadCommand>,
}

// Header

pub const MAGIC_HEADER: u32 = 0xfeedfacf;

#[derive(Serialize)]
pub struct Header {
    pub cpu_type: CPUType,
    pub cpu_sub_type: CPUSubType,
    pub file_type: FileType,
    pub no_cmds: usize,
    pub size_of_cmds: usize,
    pub flags: Vec<HeaderFlag>,
}

const CPU_ARCH_ABI64: i32 = 0x01000000;

#[repr(i32)]
#[derive(Serialize, Copy, Clone, PartialEq, Debug)]
pub enum CPUType {
    X86_64 = 7 | CPU_ARCH_ABI64,
    ARM64 = 12 | CPU_ARCH_ABI64,
}

#[repr(i32)]
#[derive(Serialize, Debug)]
pub enum CPUSubType {
    ARM(CPUARMSubType),
    X86(CPUX86SubType),
}

#[repr(i32)]
#[derive(Serialize, Debug)]
pub enum CPUARMSubType {
    All = 0,
}

#[repr(i32)]
#[derive(Serialize, Debug)]
pub enum CPUX86SubType {
    All = 0,
    All64 = 3,
}

#[repr(u32)]
#[derive(Serialize, Debug)]
pub enum FileType {
    MhObject = 0x1,   /* relocatable object file */
    MhExecuted = 0x2, /* demand paged executable file */
    MhDSYM = 0xa,     /* companion file with only debug sections */
}

#[repr(u32)]
#[derive(Serialize, Copy, Clone)]
pub enum HeaderFlag {
    MH_NOUNDEFS = 0x01, /* the object file has no undefined references */
    MH_INCRLINK = 0x02, /* the object file is the output of an incremental link against a base file
                        and can't be link edited again */
    MH_DYLDLINK = 0x4, /* the object file is input for the dynamic linker and can't be staticly
                       link edited again */
    MH_BINDATLOAD = 0x8, /* the object file's undefined references are bound by the dynamic
                         linker when loaded. */
    MH_PREBOUND = 0x10, /* the file has its dynamic undefined references prebound. */
    MH_SPLIT_SEGS = 0x20, /* the file has its read-only and read-write segments split */
    MH_LAZY_INIT = 0x40, /* the shared library init routine is to be run lazily via catching memory
                        faults to its writeable segments (obsolete) */
    MH_TWOLEVEL = 0x80,    /* the image is using two-level name space bindings */
    MH_FORCE_FLAT = 0x100, /* the executable is forcing all images to use flat name space bindings */
    MH_NOMULTIDEFS = 0x200, /* this umbrella guarantees no multiple defintions of symbols in its
                           sub-images so the two-level namespace hints can always be used. */
    MH_NOFIXPREBINDING = 0x400, /* do not have dyld notify the prebinding agent about this executable */
    MH_PREBINDABLE = 0x800, /* the binary is not prebound but can have its prebinding redone. only used
                            when MH_PREBOUND is not set. */
    MH_ALLMODSBOUND = 0x1000, /* indicates that this binary binds to all two-level namespace modules of
                              its dependent libraries. only used when MH_PREBINDABLE and MH_TWOLEVEL
                              are both set. */
    MH_SUBSECTIONS_VIA_SYMBOLS = 0x2000, /* safe to divide up the sections into sub-sections via symbols for dead
                                         code stripping */
    MH_CANONICAL = 0x4000, /* the binary has been canonicalized via the unprebind operation */
    MH_WEAK_DEFINES = 0x8000, /* the final linked image contains external weak symbols */
    MH_BINDS_TO_WEAK = 0x10000, /* the final linked image uses weak symbols */
    MH_ALLOW_STACK_EXECUTION = 0x20000, /* When this bit is set, all stacks in the task will be given stack
                                        execution privilege.  Only used in MH_EXECUTE filetypes. */
    MH_ROOT_SAFE = 0x40000, /* When this bit is set, the binary declares it is safe for use in
                            processes with uid zero */
    MH_SETUID_SAFE = 0x80000, /* When this bit is set, the binary declares it is safe for use in
                              processes when issetugid() is true */
    MH_NO_REEXPORTED_DYLIBS = 0x100000, /* When this bit is set on a dylib, the static linker does not need to
                                        examine dependent dylibs to see if any are re-exported */
    MH_PIE = 0x200000, /* When this bit is set, the OS will load the main executable at a
                       random address.  Only used in MH_EXECUTE filetypes. */
    MH_DEAD_STRIPPABLE_DYLIB = 0x400000, /* Only for use on dylibs.  When linking against a dylib that
                                         has this bit set, the static linker will automatically not create a
                                         LC_LOAD_DYLIB load command to the dylib if no symbols are being
                                         referenced from the dylib. */
    MH_HAS_TLV_DESCRIPTORS = 0x800000, /* Contains a section of type S_THREAD_LOCAL_VARIABLES */
    MH_NO_HEAP_EXECUTION = 0x1000000, /* When this bit is set, the OS will run the main executable with
                                      a non-executable heap even on platforms (e.g. i386) that don't
                                      require it. Only used in MH_EXECUTE filetypes. */
    MH_APP_EXTENSION_SAFE = 0x02000000, /* The code was linked for use in an application extension. */
    MH_NLIST_OUTOFSYNC_WITH_DYLDINFO = 0x04000000, /* The external symbols listed in the nlist symbol table do
                                                   not include all the symbols listed in the dyld info. */
    MH_SIM_SUPPORT = 0x08000000, /* Allow LC_MIN_VERSION_MACOS and LC_BUILD_VERSION load commands with
                                 the platforms macOS, macCatalyst, iOSSimulator, tvOSSimulator and
                                 watchOSSimulator. */
    MH_IMPLICIT_PAGEZERO = 0x10000000, /* main executable has no __PAGEZERO segment.  Instead, loader (xnu)
                                       will load program high and block out all memory below it. */
    MH_DYLIB_IN_CACHE = 0x80000000, /* Only for use on dylibs. When this bit is set, the dylib is part of the dyld
                                    shared cache, rather than loose in
                                    the filesystem. */
}

// Load Commands

const LC_REQ_DYLD: u32 = 0x80000000;
const LC_DYLD_INFO_ONLY: u32 = 0x22 | LC_REQ_DYLD;
const LC_MAIN: u32 = 0x28 | LC_REQ_DYLD;
const LC_DYLD_EXPORTS_TRIE: u32 = 0x33 | LC_REQ_DYLD;
const LC_DYLD_CHAINED_FIXUPS: u32 = 0x34 | LC_REQ_DYLD;

#[derive(Serialize)]
pub struct SymtabCommand {
    cmd_size: usize,
}

#[derive(Serialize)]
pub struct DsymtabCommand {
    cmd_size: usize,
}

#[derive(Serialize)]
pub struct LoadDylibCommand {
    pub cmd_size: usize,
    pub name: String,
    pub timestamp: u32,
    pub current_version: u32,
    pub compatibility_version: u32,
}

#[derive(Serialize)]
pub struct DylinkerCommand {
    cmd_size: usize,
    pub name: String,
}

#[derive(Serialize)]
pub struct Segment64Command {
    pub cmd_size: usize,
    pub name: String,
    pub vm_addr: u64,
    pub vm_size: u64,
    pub file_off: u64,
    pub file_size: u64,
    pub max_prot: i32,
    pub init_prot: i32,
    pub n_sects: u32,
    pub flags: u32, // TODO convert to Segment Flag Vec!
    pub sections: Vec<Section64>,
}

#[derive(Serialize)]
pub struct Section64 {
    pub name: String,
    pub seg_name: String,
    pub address: u64,
    pub size: u64,
    pub offset: u32,
    pub align: u32,
    pub relocation_offset: u32,
    pub n_relocations: u32,
    pub flags: u32, // TODO convert to Section Flag Vec!
}

#[derive(Serialize)]
pub struct UuidCommand {
    cmd_size: usize,
}

#[derive(Serialize)]
pub struct CodeSignatureCommand {
    cmd_size: usize,
}

#[derive(Serialize)]
pub struct BuildVersionCommand {
    cmd_size: usize,
}

#[derive(Serialize)]
pub struct FunctionStartsCommand {
    cmd_size: usize,
}

#[derive(Serialize)]
pub struct DataInCodeCommand {
    cmd_size: usize,
}

#[derive(Serialize)]
pub struct SourceVersionCommand {
    cmd_size: usize,
}

#[derive(Serialize)]
pub struct DyldInfoOnlyCommand {
    cmd_size: usize,
}

#[derive(Serialize)]
pub struct MainCommand {
    cmd_size: usize,
}

#[derive(Serialize)]
pub struct LinkeditDataCommand {
    cmd_size: usize,
}

#[derive(Serialize)]
pub enum LoadCommand {
    // 0x2
    Symtab(SymtabCommand),
    // 0xb
    Dsymtab(DsymtabCommand),
    // 0xc
    LoadDylib(LoadDylibCommand),
    // 0xe
    Dylinker(DylinkerCommand),
    // 0x19
    Segment64(Segment64Command),
    // 0x1b
    Uuid(UuidCommand),
    // 0x1d
    CodeSignature(CodeSignatureCommand),
    // 0x32
    BuildVersion(BuildVersionCommand),
    // 0x26
    FunctionStarts(FunctionStartsCommand),
    // 0x29
    DataInCode(DataInCodeCommand),
    // 0x2A
    SourceVersion(SourceVersionCommand),
    // (0x22|LC_REQ_DYLD)
    DyldInfoOnly(DyldInfoOnlyCommand),
    // (0x28|LC_REQ_DYLD)
    Main(MainCommand),
    LinkeditData(LinkeditDataCommand),
    Unknow(u32),
}

pub fn load(data: &[u8]) -> Result<Macho, String> {
    let mut reader = DataReader::new(data);
    let header = parse_header(&mut reader)?;
    let load_commands = parse_load_commands(&mut reader, header.no_cmds)?;
    Ok(Macho {
        header,
        load_commands,
    })
}

fn parse_header(reader: &mut DataReader) -> Result<Header, String> {
    let magic = reader.read_u32();
    if magic != MAGIC_HEADER {
        return Err("not a mach-o 64 file".to_string());
    }
    let cpu_type = parse_cpu_type(reader.read_i32())?;
    let cpu_sub_type = parse_cpu_sub_type(cpu_type, reader.read_i32())?;
    let file_type = parse_file_type(reader.read_u32())?;
    let no_cmds = reader.read_u32() as usize;
    let size_of_cmds = reader.read_u32() as usize;
    let flags = parse_header_flags(reader.read_u32())?;
    reader.skip(4); // reserved

    Ok(Header {
        cpu_type,
        cpu_sub_type,
        file_type,
        no_cmds,
        size_of_cmds,
        flags,
    })
}

fn parse_cpu_type(v: i32) -> Result<CPUType, String> {
    if v == CPUType::X86_64 as i32 {
        Ok(CPUType::X86_64)
    } else if v == CPUType::ARM64 as i32 {
        Ok(CPUType::ARM64)
    } else {
        Err(format!("unsupported cpu_type: 0x{:x}", v))
    }
}

fn parse_cpu_sub_type(cpu_type: CPUType, v: i32) -> Result<CPUSubType, String> {
    match cpu_type {
        CPUType::ARM64 => match v {
            0 => Ok(CPUSubType::ARM(CPUARMSubType::All)),
            _ => Err(format!("unsupported ARM64 cpu_sub_type: 0x{:x}", v)),
        },
        CPUType::X86_64 => match v {
            0 => Ok(CPUSubType::X86(CPUX86SubType::All)),
            3 => Ok(CPUSubType::X86(CPUX86SubType::All64)),
            _ => Err(format!("unsupported X86_64 cpu_sub_type: 0x{:x}", v)),
        },
    }
}

fn parse_file_type(v: u32) -> Result<FileType, String> {
    match v {
        0x1 => Ok(FileType::MhObject),
        0x2 => Ok(FileType::MhExecuted),
        0xa => Ok(FileType::MhDSYM),
        _ => Err(format!("unsupported file_type: 0x{:x}", v)),
    }
}

fn parse_header_flags(v: u32) -> Result<Vec<HeaderFlag>, String> {
    let mut result = Vec::new();
    let r = &mut result;
    h_flag(v, HeaderFlag::MH_NOUNDEFS, r);
    h_flag(v, HeaderFlag::MH_INCRLINK, r);
    h_flag(v, HeaderFlag::MH_DYLDLINK, r);
    h_flag(v, HeaderFlag::MH_BINDATLOAD, r);
    h_flag(v, HeaderFlag::MH_PREBOUND, r);
    h_flag(v, HeaderFlag::MH_SPLIT_SEGS, r);
    h_flag(v, HeaderFlag::MH_LAZY_INIT, r);
    h_flag(v, HeaderFlag::MH_TWOLEVEL, r);
    h_flag(v, HeaderFlag::MH_FORCE_FLAT, r);
    h_flag(v, HeaderFlag::MH_NOMULTIDEFS, r);
    h_flag(v, HeaderFlag::MH_NOFIXPREBINDING, r);
    h_flag(v, HeaderFlag::MH_PREBINDABLE, r);
    h_flag(v, HeaderFlag::MH_ALLMODSBOUND, r);
    h_flag(v, HeaderFlag::MH_SUBSECTIONS_VIA_SYMBOLS, r);
    h_flag(v, HeaderFlag::MH_CANONICAL, r);
    h_flag(v, HeaderFlag::MH_WEAK_DEFINES, r);
    h_flag(v, HeaderFlag::MH_BINDS_TO_WEAK, r);
    h_flag(v, HeaderFlag::MH_ALLOW_STACK_EXECUTION, r);
    h_flag(v, HeaderFlag::MH_ROOT_SAFE, r);
    h_flag(v, HeaderFlag::MH_SETUID_SAFE, r);
    h_flag(v, HeaderFlag::MH_NO_REEXPORTED_DYLIBS, r);
    h_flag(v, HeaderFlag::MH_PIE, r);
    h_flag(v, HeaderFlag::MH_DEAD_STRIPPABLE_DYLIB, r);
    h_flag(v, HeaderFlag::MH_HAS_TLV_DESCRIPTORS, r);
    h_flag(v, HeaderFlag::MH_NO_HEAP_EXECUTION, r);
    h_flag(v, HeaderFlag::MH_APP_EXTENSION_SAFE, r);
    h_flag(v, HeaderFlag::MH_NLIST_OUTOFSYNC_WITH_DYLDINFO, r);
    h_flag(v, HeaderFlag::MH_SIM_SUPPORT, r);
    h_flag(v, HeaderFlag::MH_IMPLICIT_PAGEZERO, r);
    h_flag(v, HeaderFlag::MH_DYLIB_IN_CACHE, r);
    Ok(result)
}

fn h_flag(v: u32, flag: HeaderFlag, result: &mut Vec<HeaderFlag>) {
    if (v & (flag as u32)) != 0 {
        result.push(flag)
    }
}

fn parse_load_commands(
    reader: &mut DataReader,
    no_cmds: usize,
) -> Result<Vec<LoadCommand>, String> {
    let mut commands = Vec::with_capacity(no_cmds);
    for _ in 0..no_cmds {
        let cmd = reader.read_u32();
        let cmd_size = reader.read_u32() as usize;
        let command = match cmd {
            0x2 => parse_cmd_symtab(reader, cmd_size),
            0xb => parse_cmd_dsymtab(reader, cmd_size),
            0xc => parse_cmd_load_dylib(reader, cmd_size),
            0xe => parse_cmd_dylinker(reader, cmd_size),
            0x19 => parse_cmd_segment_64(reader, cmd_size),
            0x1b => parse_cmd_uuid(reader, cmd_size),
            0x1d => parse_cmd_code_signature(reader, cmd_size),
            0x26 => parse_cmd_function_starts(reader, cmd_size),
            0x2a => parse_cmd_source_version(reader, cmd_size),
            0x29 => parse_cmd_data_in_code(reader, cmd_size),
            0x32 => parse_cmd_build_version(reader, cmd_size),
            LC_DYLD_INFO_ONLY => parse_cmd_dyld_info_only(reader, cmd_size),
            LC_MAIN => parse_cmd_main(reader, cmd_size),
            LC_DYLD_EXPORTS_TRIE => parse_cmd_dyld_exports_trie(reader, cmd_size),
            LC_DYLD_CHAINED_FIXUPS => parse_cmd_dyld_chained_fixups(reader, cmd_size),
            _ => {
                reader.skip(cmd_size - 8);
                Ok(LoadCommand::Unknow(cmd))
            }
        }?;
        commands.push(command);
    }
    Ok(commands)
}

fn parse_cmd_symtab(reader: &mut DataReader, cmd_size: usize) -> Result<LoadCommand, String> {
    reader.skip(cmd_size - 8);
    Ok(LoadCommand::Symtab(SymtabCommand { cmd_size }))
}

fn parse_cmd_dsymtab(reader: &mut DataReader, cmd_size: usize) -> Result<LoadCommand, String> {
    reader.skip(cmd_size - 8);
    Ok(LoadCommand::Dsymtab(DsymtabCommand { cmd_size }))
}

fn parse_cmd_load_dylib(reader: &mut DataReader, cmd_size: usize) -> Result<LoadCommand, String> {
    reader.skip(4); //name offset, derived from cmd_size
    let timestamp = reader.read_u32();
    let current_version = reader.read_u32();
    let compatibility_version = reader.read_u32();
    let name = clean_string(&reader.read_utf8_string(cmd_size - (6 * 4)));
    Ok(LoadCommand::LoadDylib(LoadDylibCommand {
        cmd_size,
        name,
        timestamp,
        current_version,
        compatibility_version,
    }))
}

fn parse_cmd_dylinker(reader: &mut DataReader, cmd_size: usize) -> Result<LoadCommand, String> {
    let name_offset = reader.read_i32() as usize;
    assert_eq!(12, name_offset);
    let name = clean_string(&reader.read_utf8_string(cmd_size - 12));
    Ok(LoadCommand::Dylinker(DylinkerCommand { cmd_size, name }))
}

fn parse_cmd_segment_64(reader: &mut DataReader, cmd_size: usize) -> Result<LoadCommand, String> {
    let start = reader.offset();

    let name = clean_string(&reader.read_utf8_string(16));
    let vm_addr = reader.read_u64();
    let vm_size = reader.read_u64();
    let file_off = reader.read_u64();
    let file_size = reader.read_u64();
    let max_prot = reader.read_i32();
    let init_prot = reader.read_i32();
    let n_sects = reader.read_u32();
    let flags = reader.read_u32();

    let mut sections = Vec::with_capacity(n_sects as usize);
    for _ in 0..n_sects {
        let sec = parse_section_64(reader)?;
        sections.push(sec);
    }

    //assure reader is at the end of the load command
    reader.skip(cmd_size - (reader.offset() - start) - 8);

    Ok(LoadCommand::Segment64(Segment64Command {
        cmd_size,
        name,
        vm_addr,
        vm_size,
        file_off,
        file_size,
        max_prot,
        init_prot,
        n_sects,
        flags,
        sections,
    }))
}

fn parse_section_64(reader: &mut DataReader) -> Result<Section64, String> {
    let name = clean_string(&reader.read_utf8_string(16));
    let seg_name = clean_string(&reader.read_utf8_string(16));
    let address = reader.read_u64();
    let size = reader.read_u64();
    let offset = reader.read_u32();
    let align = reader.read_u32();
    let relocation_offset = reader.read_u32();
    let n_relocations = reader.read_u32();
    let flags = reader.read_u32();
    reader.skip(12); //reserved1, reserved2, reserved3 u32
    Ok(Section64 {
        name,
        seg_name,
        address,
        size,
        offset,
        align,
        relocation_offset,
        n_relocations,
        flags,
    })
}

fn parse_cmd_uuid(reader: &mut DataReader, cmd_size: usize) -> Result<LoadCommand, String> {
    reader.skip(cmd_size - 8);
    Ok(LoadCommand::Uuid(UuidCommand { cmd_size }))
}

fn parse_cmd_code_signature(
    reader: &mut DataReader,
    cmd_size: usize,
) -> Result<LoadCommand, String> {
    reader.skip(cmd_size - 8);
    Ok(LoadCommand::CodeSignature(CodeSignatureCommand {
        cmd_size,
    }))
}

fn parse_cmd_build_version(
    reader: &mut DataReader,
    cmd_size: usize,
) -> Result<LoadCommand, String> {
    reader.skip(cmd_size - 8);
    Ok(LoadCommand::BuildVersion(BuildVersionCommand { cmd_size }))
}

fn parse_cmd_function_starts(
    reader: &mut DataReader,
    cmd_size: usize,
) -> Result<LoadCommand, String> {
    reader.skip(cmd_size - 8);
    Ok(LoadCommand::FunctionStarts(FunctionStartsCommand {
        cmd_size,
    }))
}

fn parse_cmd_source_version(
    reader: &mut DataReader,
    cmd_size: usize,
) -> Result<LoadCommand, String> {
    reader.skip(cmd_size - 8);
    Ok(LoadCommand::SourceVersion(SourceVersionCommand {
        cmd_size,
    }))
}

fn parse_cmd_data_in_code(reader: &mut DataReader, cmd_size: usize) -> Result<LoadCommand, String> {
    reader.skip(cmd_size - 8);
    Ok(LoadCommand::DataInCode(DataInCodeCommand { cmd_size }))
}

fn parse_cmd_dyld_info_only(
    reader: &mut DataReader,
    cmd_size: usize,
) -> Result<LoadCommand, String> {
    reader.skip(cmd_size - 8);
    Ok(LoadCommand::DyldInfoOnly(DyldInfoOnlyCommand { cmd_size }))
}

fn parse_cmd_main(reader: &mut DataReader, cmd_size: usize) -> Result<LoadCommand, String> {
    reader.skip(cmd_size - 8);
    Ok(LoadCommand::Main(MainCommand { cmd_size }))
}

fn parse_cmd_dyld_exports_trie(
    reader: &mut DataReader,
    cmd_size: usize,
) -> Result<LoadCommand, String> {
    reader.skip(cmd_size - 8);
    Ok(LoadCommand::LinkeditData(LinkeditDataCommand { cmd_size }))
}

fn parse_cmd_dyld_chained_fixups(
    reader: &mut DataReader,
    cmd_size: usize,
) -> Result<LoadCommand, String> {
    reader.skip(cmd_size - 8);
    Ok(LoadCommand::LinkeditData(LinkeditDataCommand { cmd_size }))
}

// helper

pub struct DataReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl DataReader<'_> {
    pub fn new(data: &[u8]) -> DataReader<'_> {
        DataReader::new_with_offset(data, 0)
    }

    pub fn new_with_offset(data: &[u8], offset: usize) -> DataReader<'_> {
        DataReader { data, offset }
    }
}

impl DataReader<'_> {
    pub fn read_utf8_string(&mut self, size: usize) -> String {
        let str =
            String::from_utf8_lossy(&self.data[self.offset..(self.offset + size)]).to_string();
        self.offset += size;
        str
    }

    pub fn read_u64(&mut self) -> u64 {
        let u = u64::from_le_bytes(
            self.data[self.offset..(self.offset + 8)]
                .try_into()
                .unwrap(),
        );
        self.offset += 8;
        u
    }

    pub fn read_u32(&mut self) -> u32 {
        let u = u32::from_le_bytes(
            self.data[self.offset..(self.offset + 4)]
                .try_into()
                .unwrap(),
        );
        self.offset += 4;
        u
    }

    pub fn read_i32(&mut self) -> i32 {
        let i = i32::from_le_bytes(
            self.data[self.offset..(self.offset + 4)]
                .try_into()
                .unwrap(),
        );
        self.offset += 4;
        i
    }

    pub fn read_u16(&mut self) -> u16 {
        let u = u16::from_le_bytes(
            self.data[self.offset..(self.offset + 2)]
                .try_into()
                .unwrap(),
        );
        self.offset += 2;
        u
    }

    pub fn read_i16(&mut self) -> i16 {
        let i = i16::from_le_bytes(
            self.data[self.offset..(self.offset + 2)]
                .try_into()
                .unwrap(),
        );
        self.offset += 2;
        i
    }

    pub fn read_u8(&mut self) -> u8 {
        let u = self.data[self.offset];
        self.offset += 1;
        u
    }

    pub fn read_bool(&mut self) -> bool {
        let u = self.read_u16();
        u != 0
    }

    // returns a slice over the bytes that were not read so far
    pub fn unread_bytes(&self) -> &[u8] {
        &self.data[self.offset..]
    }

    pub fn slice(&self, start: usize, end: usize) -> &[u8] {
        &self.data[start..end]
    }

    pub fn skip(&mut self, bytes: usize) {
        self.offset += bytes;
    }

    pub fn offset(&self) -> usize {
        return self.offset;
    }
}

fn clean_string(str: &str) -> String {
    str.replace('\0', "")
}
