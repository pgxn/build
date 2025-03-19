use super::*;
use std::str;

#[test]
fn line_writer() {
    let mut buf: Vec<u8> = Vec::new();
    let mut lw = LineWriter::new(&mut buf);
    assert!(lw.write_line("hello").is_ok());
    assert!(lw.write("yes".as_bytes()).is_ok());
    assert!(lw.flush().is_ok());
    let res = str::from_utf8(buf.as_slice()).unwrap();
    assert_eq!("hello\nyes", res);

    // Write a couple of lines with a boxed LineWriter.
    let mut buf: Vec<u8> = Vec::new();
    let mut lw = Box::new(LineWriter::new(&mut buf));
    assert!(lw.write_line("🤘🏻 Rock on gold dust woman").is_ok());
    assert!(lw.write_line("🐦‍⬛ Crows are rad.").is_ok());
    let res = str::from_utf8(buf.as_slice()).unwrap();
    assert_eq!("🤘🏻 Rock on gold dust woman\n🐦‍⬛ Crows are rad.\n", res);
}

#[test]
fn color_line() {
    let mut buf: Vec<u8> = Vec::new();
    let style = Style::new().green();
    let mut lw = ColorLine::new(&mut buf, style);
    assert!(lw.write_line("hello").is_ok());
    assert!(lw.write("yes".as_bytes()).is_ok());
    assert!(lw.flush().is_ok());
    let res = str::from_utf8(buf.as_slice()).unwrap();
    assert_eq!(format!("{}\nyes", style.style("hello")), res);

    // Write a couple of lines with a boxed LineWriter.
    let mut buf: Vec<u8> = Vec::new();
    let style = Style::new().red();
    let mut lw = Box::new(ColorLine::new(&mut buf, style));
    assert!(lw.write_line("🤘🏻 Rock on gold dust woman").is_ok());
    assert!(lw.write_line("🐦‍⬛ Crows are rad.").is_ok());
    let res = str::from_utf8(buf.as_slice()).unwrap();
    assert_eq!(
        format!(
            "{}\n{}\n",
            style.style("🤘🏻 Rock on gold dust woman"),
            style.style("🐦‍⬛ Crows are rad.")
        ),
        res
    );
}
