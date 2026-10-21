/// Makes typed or spoken words comparable: lowercase and without accents,
/// so "Mañana", "manana" and "MAÑANA" are the same word.
pub struct TextNormalizer;

impl TextNormalizer {
    pub fn normalize(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(Self::without_accent)
            .collect()
    }

    fn without_accent(character: char) -> char {
        match character {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            other => other,
        }
    }
}
