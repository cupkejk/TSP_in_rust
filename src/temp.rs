fn main() {
    let cities: Vec<[f32; 2]> = Vec::new();  // cities jest właścicielem danych
    let tour = cities;  // przeniesienie własności (move)
    // cities już nie istnieje — kompilator zgłosi błąd przy próbie użycia!
}