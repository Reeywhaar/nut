use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_pages() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("nut")?;

    cmd.arg("pages")
        .arg("--path")
        .arg("./test_data/freelist.db");

    cmd.assert().success().stdout(
        "[32m    ID Type              Count Overflow[0m
------ --------------- ------- --------
     0 Flags(META)           0        0
     1 Flags(META)           0        0
     2 Flags(LEAVES)         1        0
     3 Flags(LEAVES)        13        0
     4 Flags(FREELIST)      15        0
     5 free
     6 free
     7 free
     8 free
     9 free
    10 Flags(LEAVES)         4        3
    14 free
    15 free
    16 free
    17 free
    18 free
    19 free
    20 free
    21 free
    22 free
    23 free
",
    );

    Ok(())
}
