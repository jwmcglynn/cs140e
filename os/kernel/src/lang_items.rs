use console::kprintln;

#[no_mangle]
#[cfg(not(test))]
#[lang = "panic_fmt"]
pub extern fn panic_fmt(fmt: ::std::fmt::Arguments, file: &'static str, line: u32) -> ! {
    kprintln!(r#"                                   88"#);
    kprintln!(r#"                                   """#);
    kprintln!("");
    kprintln!(r#"8b,dPPYba,  ,adPPYYba, 8b,dPPYba,  88  ,adPPYba,"#);
    kprintln!(r#"88P'    "8a ""     `Y8 88P'   `"8a 88 a8"     """#);
    kprintln!(r#"88       d8 ,adPPPPP88 88       88 88 8b"#);
    kprintln!(r#"88b,   ,a8" 88,    ,88 88       88 88 "8a,   ,aa"#);
    kprintln!(r#"88`YbbdP"'  `"8bbdP"Y8 88       88 88  `"Ybbd8"'"#);
    kprintln!(r#"88"#);
    kprintln!(r#"88"#);
    kprintln!("");
    kprintln!("");
    kprintln!("{}:{}", file, line);
    kprintln!("{}", fmt);

    loop { unsafe { asm!("wfe") } }
}

#[cfg(not(test))] #[lang = "eh_personality"] pub extern fn eh_personality() {}
