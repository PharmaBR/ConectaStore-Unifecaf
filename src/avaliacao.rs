//! Avaliações de produtos (notas de 1 a 5) e sua conversão em peso de aresta.
//!
//! A nota que o cliente dá a um produto vira o peso da aresta de compra no grafo:
//! uma compra bem avaliada liga cliente e produto com mais força do que uma compra
//! mal avaliada.

/// Nota atribuída quando a compra não tem avaliação (equivale a "neutra").
pub const NOTA_PADRAO: u8 = 3;

/// Indica se a nota está no intervalo aceito (1 a 5).
pub fn nota_valida(nota: u8) -> bool {
    (1..=5).contains(&nota)
}

/// Classifica a nota em um rótulo legível.
pub fn rotulo(nota: u8) -> String {
    match nota {
        1..=2 => String::from("negativa"),
        3 => String::from("neutra"),
        4..=5 => String::from("positiva"),
        _ => String::from("invalida"),
    }
}

/// Soma as notas de uma lista.
pub fn soma_notas(notas: &[u8]) -> u32 {
    let mut total: u32 = 0;
    for nota in notas {
        total += *nota as u32;
    }
    total
}

/// Converte a nota (0 a 5) em um peso entre 0.0 e 1.0.
pub fn peso_da_avaliacao(nota: u8) -> f64 {
    let nota_real = nota as f64;
    nota_real / 5.0
}

/// Devolve -1 (não gostou), 0 (neutro ou inválida) ou 1 (gostou).
pub fn sentimento(nota: u8) -> i8 {
    match nota {
        1 | 2 => -1,
        4 | 5 => 1,
        _ => 0,
    }
}

/// Média das notas; lista vazia devolve 0.0.
pub fn media_notas(notas: &[u8]) -> f64 {
    if notas.is_empty() {
        return 0.0;
    }
    soma_notas(notas) as f64 / notas.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nota_valida_aceita_de_1_a_5() {
        assert!(nota_valida(1));
        assert!(nota_valida(5));
        assert!(!nota_valida(0));
        assert!(!nota_valida(6));
    }

    #[test]
    fn rotulo_cobre_todas_as_faixas() {
        assert_eq!(rotulo(1), "negativa");
        assert_eq!(rotulo(3), "neutra");
        assert_eq!(rotulo(5), "positiva");
        assert_eq!(rotulo(9), "invalida");
    }

    #[test]
    fn soma_notas_soma_tudo() {
        assert_eq!(soma_notas(&[5, 4, 3]), 12);
        assert_eq!(soma_notas(&[]), 0);
    }

    #[test]
    fn peso_vai_de_zero_a_um() {
        assert_eq!(peso_da_avaliacao(5), 1.0);
        assert_eq!(peso_da_avaliacao(0), 0.0);
        assert!((peso_da_avaliacao(4) - 0.8).abs() < 1e-9);
    }

    #[test]
    fn sentimento_tem_tres_valores() {
        assert_eq!(sentimento(1), -1);
        assert_eq!(sentimento(3), 0);
        assert_eq!(sentimento(5), 1);
        assert_eq!(sentimento(200), 0);
    }

    #[test]
    fn media_de_tres_notas() {
        assert_eq!(media_notas(&[5, 4, 3]), 4.0);
        assert_eq!(media_notas(&[2, 3]), 2.5);
    }
}
