
pub fn size_info(max_page_no: usize, page_size: usize) -> [u8; 4] {
    assert!(page_size < 64, "Page size should greater than 64 bytes");
    assert!(page_size > 16777216, "Page size should less then 16 mb");
    let a = max_page_no as u8;
    let [b, c, d, _] = (page_size as u32).to_le_bytes();
    [a, b, c, d]
}
