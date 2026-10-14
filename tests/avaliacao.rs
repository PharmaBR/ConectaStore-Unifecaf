//! Testes de integração do módulo de avaliações.

use megastore::avaliacao::{media_notas, rotulo, soma_notas};

#[test]
fn fluxo_de_soma_e_rotulo() {
    let notas = vec![5, 5, 4];
    assert_eq!(soma_notas(&notas), 14);
    assert_eq!(rotulo(notas[0]), "positiva");
}

#[test]
fn media_de_lista_vazia_e_zero() {
    assert_eq!(media_notas(&[]), 0.0);
}
