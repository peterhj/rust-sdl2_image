#![crate_name="sdl2_image"]
#![crate_type = "lib"]

extern crate sdl2;
extern crate sdl2_sys as sys;

#[macro_use]
extern crate bitflags;

use std::os::raw::{c_int, c_char};
use std::ffi::CString;
use std::path::Path;
use sdl2::surface::Surface;
use sdl2::render::Texture;
use sdl2::render::Renderer;
use sdl2::rwops::RWops;
use sdl2::version::Version;
use sdl2::get_error;

// Setup linking for all targets.
#[cfg(target_os="macos")]
mod mac {
    #[cfg(mac_framework)]
    #[link(kind="framework", name="SDL2_image")]
    extern {}

    #[cfg(not(mac_framework))]
    #[link(name="SDL2_image")]
    extern {}
}

#[cfg(any(target_os="windows", target_os="linux", target_os="freebsd"))]
mod others {
    #[link(name="SDL2_image", kind="static")]
    extern {}
}

#[allow(non_camel_case_types, dead_code)]
mod ffi;

/// InitFlags are passed to init() to control which subsystem
/// functionality to load.
bitflags! {
    pub flags InitFlag : u32 {
        const INIT_JPG  = ::ffi::IMG_INIT_JPG as u32,
        const INIT_PNG  = ::ffi::IMG_INIT_PNG as u32,
        const INIT_TIF  = ::ffi::IMG_INIT_TIF as u32,
        const INIT_WEBP = ::ffi::IMG_INIT_WEBP as u32
    }
}

// This is used for error message for init
impl ToString for InitFlag {
    fn to_string(&self) -> String {
        let mut string = "".to_string();
        if self.contains(INIT_JPG) {
            string = string + &"INIT_JPG ".to_string();
        }
        if self.contains(INIT_PNG) {
            string = string + &"INIT_PNG ".to_string();
        }
        if self.contains(INIT_TIF) {
            string = string + &"INIT_TIF ".to_string();
        }
        if self.contains(INIT_WEBP) {
            string = string + &"INIT_WEBP ".to_string();
        }
        string
    }
}


/// Static method extensions for creating Surfaces
pub trait LoadSurface: Sized {
    // Self is only returned here to type hint to the compiler.
    // The syntax for type hinting in this case is not yet defined.
    // The intended return value is Result<~Surface, String>.
    fn from_file(filename: &Path) -> Result<Self, String>;
    fn from_xpm_array(xpm: *const *const i8) -> Result<Self, String>;
}

/// Method extensions to Surface for saving to disk
pub trait SaveSurface {
    fn save(&self, filename: &Path) -> Result<(), String>;
    fn save_rw(&self, dst: &mut RWops) -> Result<(), String>;
}

impl<'a> LoadSurface for Surface<'a> {
    fn from_file(filename: &Path) -> Result<Surface<'a>, String> {
        //! Loads an SDL Surface from a file
        unsafe {
            let raw = ffi::IMG_Load(CString::new(filename.to_str().unwrap()).unwrap().as_ptr() as *const _);
            if (raw as *mut ()).is_null() {
                Err(get_error())
            } else {
                Ok(Surface::from_ll(raw))
            }
        }
    }

    fn from_xpm_array(xpm: *const *const i8) -> Result<Surface<'a>, String> {
        //! Loads an SDL Surface from XPM data
        unsafe {
            let raw = ffi::IMG_ReadXPMFromArray(xpm as *const *const c_char);
            if (raw as *mut ()).is_null() {
                Err(get_error())
            } else {
                Ok(Surface::from_ll(raw))
            }
        }
    }
}

impl<'a> SaveSurface for Surface<'a> {
    fn save(&self, filename: &Path) -> Result<(), String> {
        //! Saves an SDL Surface to a file
        unsafe {
            let status = ffi::IMG_SavePNG(self.raw(), CString::new(filename.to_str().unwrap()).unwrap().as_ptr() as *const _);
            if status != 0 {
                Err(get_error())
            } else {
                Ok(())
            }
        }
    }

    fn save_rw(&self, dst: &mut RWops) -> Result<(), String> {
        //! Saves an SDL Surface to an RWops
        unsafe {
            let status = ffi::IMG_SavePNG_RW(self.raw(), dst.raw(), 0);

            if status != 0 {
                Err(get_error())
            } else {
                Ok(())
            }
        }
    }
}

/// Method extensions for creating Textures from a Renderer
pub trait LoadTexture {
    fn load_texture(&self, filename: &Path) -> Result<Texture, String>;
}

impl<'a> LoadTexture for Renderer<'a> {
    fn load_texture(&self, filename: &Path) -> Result<Texture, String> {
        //! Loads an SDL Texture from a file
        unsafe {
            let raw = ffi::IMG_LoadTexture(self.raw(), CString::new(filename.to_str().unwrap()).unwrap().as_ptr() as *const _);
            if (raw as *mut ()).is_null() {
                Err(get_error())
            } else {
                Ok(Texture::from_ll(self, raw))
            }
        }
    }
}

/// Context manager for sdl2_image to manage quiting. Can't do much with it but
/// keep it alive while you are using it.
pub struct Sdl2ImageContext;

impl Drop for Sdl2ImageContext {
    fn drop(&mut self) {
        unsafe { ffi::IMG_Quit(); }
    }
}

pub fn init(flags: InitFlag) -> Result<Sdl2ImageContext, String> {
    //! Initializes SDL2_image with InitFlags.
    //! If not every flag is set it returns an error
    let return_flags = unsafe {
        let used = ffi::IMG_Init(flags.bits() as c_int);
        InitFlag::from_bits_truncate(used as u32)
    };
    if !flags.intersects(return_flags) {
        // According to docs, error message text is not always set
        if get_error() == "" {
            let un_init_flags = return_flags ^ flags;
            let error_str = &("Could not init: ".to_string() + &un_init_flags.to_string());
            let _ = sdl2::set_error(error_str);
        }
        Err(get_error())
    } else {
        Ok(Sdl2ImageContext)
    }
}

pub fn get_linked_version() -> Version {
    //! Returns the version of the dynamically linked SDL_image library
    unsafe {
        Version::from_ll(*ffi::IMG_Linked_Version())
    }
}

#[inline]
fn to_surface_result<'a>(raw: *mut sys::surface::SDL_Surface) -> Result<Surface<'a>, String> {
    if (raw as *mut ()).is_null() {
        Err(get_error())
    } else {
        unsafe { Ok(Surface::from_ll(raw)) }
    }
}

pub trait ImageRWops {
    /// load as a surface. except TGA
    fn load(&self) -> Result<Surface, String>;
    /// load as a surface. This can load all supported image formats.
    fn load_typed(&self, _type: &str) -> Result<Surface, String>;

    fn load_cur(&self) -> Result<Surface, String>;
    fn load_ico(&self) -> Result<Surface, String>;
    fn load_bmp(&self) -> Result<Surface, String>;
    fn load_pnm(&self) -> Result<Surface, String>;
    fn load_xpm(&self) -> Result<Surface, String>;
    fn load_xcf(&self) -> Result<Surface, String>;
    fn load_pcx(&self) -> Result<Surface, String>;
    fn load_gif(&self) -> Result<Surface, String>;
    fn load_jpg(&self) -> Result<Surface, String>;
    fn load_tif(&self) -> Result<Surface, String>;
    fn load_png(&self) -> Result<Surface, String>;
    fn load_tga(&self) -> Result<Surface, String>;
    fn load_lbm(&self) -> Result<Surface, String>;
    fn load_xv(&self)  -> Result<Surface, String>;
    fn load_webp(&self) -> Result<Surface, String>;

    fn is_cur(&self) -> bool;
    fn is_ico(&self) -> bool;
    fn is_bmp(&self) -> bool;
    fn is_pnm(&self) -> bool;
    fn is_xpm(&self) -> bool;
    fn is_xcf(&self) -> bool;
    fn is_pcx(&self) -> bool;
    fn is_gif(&self) -> bool;
    fn is_jpg(&self) -> bool;
    fn is_tif(&self) -> bool;
    fn is_png(&self) -> bool;
    fn is_lbm(&self) -> bool;
    fn is_xv(&self)  -> bool;
    fn is_webp(&self) -> bool;
}

impl<'a> ImageRWops for RWops<'a> {
    fn load(&self) -> Result<Surface, String> {
        let raw = unsafe {
            ffi::IMG_Load_RW(self.raw(), 0)
        };
        to_surface_result(raw)
    }
    fn load_typed(&self, _type: &str) -> Result<Surface, String> {
        let raw = unsafe {
            ffi::IMG_LoadTyped_RW(self.raw(), 0, CString::new(_type.as_bytes()).unwrap().as_ptr() as *const _)
        };
        to_surface_result(raw)
    }

    fn load_cur(&self) -> Result<Surface, String> {
        let raw = unsafe { ffi::IMG_LoadCUR_RW(self.raw()) };
        to_surface_result(raw)
    }
    fn load_ico(&self) -> Result<Surface, String> {
        let raw = unsafe { ffi::IMG_LoadICO_RW(self.raw()) };
        to_surface_result(raw)
    }
    fn load_bmp(&self) -> Result<Surface, String> {
        let raw = unsafe { ffi::IMG_LoadBMP_RW(self.raw()) };
        to_surface_result(raw)
    }
    fn load_pnm(&self) -> Result<Surface, String> {
        let raw = unsafe { ffi::IMG_LoadPNM_RW(self.raw()) };
        to_surface_result(raw)
    }
    fn load_xpm(&self) -> Result<Surface, String> {
        let raw = unsafe { ffi::IMG_LoadXPM_RW(self.raw()) };
        to_surface_result(raw)
    }
    fn load_xcf(&self) -> Result<Surface, String> {
        let raw = unsafe { ffi::IMG_LoadXCF_RW(self.raw()) };
        to_surface_result(raw)
    }
    fn load_pcx(&self) -> Result<Surface, String> {
        let raw = unsafe { ffi::IMG_LoadPCX_RW(self.raw()) };
        to_surface_result(raw)
    }
    fn load_gif(&self) -> Result<Surface, String> {
        let raw = unsafe { ffi::IMG_LoadGIF_RW(self.raw()) };
        to_surface_result(raw)
    }
    fn load_jpg(&self) -> Result<Surface, String> {
        let raw = unsafe { ffi::IMG_LoadJPG_RW(self.raw()) };
        to_surface_result(raw)
    }
    fn load_tif(&self) -> Result<Surface, String> {
        let raw = unsafe { ffi::IMG_LoadTIF_RW(self.raw()) };
        to_surface_result(raw)
    }
    fn load_png(&self) -> Result<Surface, String> {
        let raw = unsafe { ffi::IMG_LoadPNG_RW(self.raw()) };
        to_surface_result(raw)
    }
    fn load_tga(&self) -> Result<Surface, String> {
        let raw = unsafe { ffi::IMG_LoadTGA_RW(self.raw()) };
        to_surface_result(raw)
    }
    fn load_lbm(&self) -> Result<Surface, String> {
        let raw = unsafe { ffi::IMG_LoadLBM_RW(self.raw()) };
        to_surface_result(raw)
    }
    fn load_xv(&self)  -> Result<Surface, String> {
        let raw = unsafe { ffi::IMG_LoadXV_RW(self.raw()) };
        to_surface_result(raw)
    }
    fn load_webp(&self)  -> Result<Surface, String> {
        let raw = unsafe { ffi::IMG_LoadWEBP_RW(self.raw()) };
        to_surface_result(raw)
    }

    fn is_cur(&self) -> bool {
        unsafe { ffi::IMG_isCUR(self.raw()) == 1 }
    }
    fn is_ico(&self) -> bool {
        unsafe { ffi::IMG_isICO(self.raw()) == 1 }
    }
    fn is_bmp(&self) -> bool {
        unsafe { ffi::IMG_isBMP(self.raw()) == 1 }
    }
    fn is_pnm(&self) -> bool {
        unsafe { ffi::IMG_isPNM(self.raw()) == 1 }
    }
    fn is_xpm(&self) -> bool {
        unsafe { ffi::IMG_isXPM(self.raw()) == 1 }
    }
    fn is_xcf(&self) -> bool {
        unsafe { ffi::IMG_isXCF(self.raw()) == 1 }
    }
    fn is_pcx(&self) -> bool {
        unsafe { ffi::IMG_isPCX(self.raw()) == 1 }
    }
    fn is_gif(&self) -> bool {
        unsafe { ffi::IMG_isGIF(self.raw()) == 1 }
    }
    fn is_jpg(&self) -> bool {
        unsafe { ffi::IMG_isJPG(self.raw()) == 1 }
    }
    fn is_tif(&self) -> bool {
        unsafe { ffi::IMG_isTIF(self.raw()) == 1 }
    }
    fn is_png(&self) -> bool {
        unsafe { ffi::IMG_isPNG(self.raw()) == 1 }
    }
    fn is_lbm(&self) -> bool {
        unsafe { ffi::IMG_isLBM(self.raw()) == 1 }
    }
    fn is_xv(&self)  -> bool {
        unsafe { ffi::IMG_isXV(self.raw())  == 1 }
    }
    fn is_webp(&self) -> bool {
        unsafe { ffi::IMG_isWEBP(self.raw())  == 1 }
    }
}
