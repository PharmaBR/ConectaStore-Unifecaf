//! Testes de integração: exercitam o sistema pela API pública, como um usuário
//! externo da biblioteca faria.

use std::collections::HashSet;

use megastore::catalogo::Catalogo;
use megastore::dados::{catalogo_exemplo, gerar_catalogo};
use megastore::modelo::ErroCatalogo;
use megastore::recomendacao::{recomendar_para_cliente, recomendar_para_produto};

#[test]
fn fluxo_completo_cadastro_compra_e_recomendacao() {
    let mut c = Catalogo::new();
    c.cadastrar_categoria(1, "Informática").unwrap();
    c.cadastrar_produto(10, "Notebook", 1, 4000.0).unwrap();
    c.cadastrar_produto(11, "Mouse", 1, 90.0).unwrap();
    c.cadastrar_produto(12, "Teclado", 1, 150.0).unwrap();
    c.cadastrar_cliente(1, "Ana").unwrap();
    c.cadastrar_cliente(2, "Bia").unwrap();

    c.registrar_compra(1, 10, Some(5)).unwrap();
    c.registrar_compra(2, 10, Some(5)).unwrap();
    c.registrar_compra(2, 12, Some(4)).unwrap();

    // Consulta de produto
    assert_eq!(
        c.buscar_produto(12).map(|p| p.nome.as_str()),
        Some("Teclado")
    );

    // Ana comprou o Notebook, como a Bia, que também comprou o Teclado.
    let recs = recomendar_para_cliente(&c, 1, 5).unwrap();
    assert_eq!(recs[0].produto_id, 12);
    assert!(recs.iter().all(|r| r.produto_id != 10));
}

#[test]
fn erros_de_dominio_chegam_a_quem_chama() {
    let mut c = Catalogo::new();
    assert_eq!(
        c.cadastrar_produto(1, "Sem categoria", 9, 1.0),
        Err(ErroCatalogo::CategoriaNaoEncontrada(9))
    );
    assert_eq!(
        recomendar_para_cliente(&c, 5, 3),
        Err(ErroCatalogo::ClienteNaoEncontrado(5))
    );
}

#[test]
fn catalogo_de_exemplo_recomenda_para_todos_sem_repetir_nem_reoferecer() {
    let c = catalogo_exemplo().unwrap();
    for cliente in c.ids_clientes() {
        let recs = recomendar_para_cliente(&c, cliente, 10).unwrap();
        let unicos: HashSet<_> = recs.iter().map(|r| r.produto_id).collect();
        assert_eq!(
            unicos.len(),
            recs.len(),
            "duplicata para o cliente {cliente}"
        );
        for r in &recs {
            assert!(
                !c.ja_comprou(cliente, r.produto_id),
                "reoferta para o cliente {cliente}"
            );
        }
    }
}

#[test]
fn catalogo_gerado_respeita_as_regras_em_escala() {
    let c = gerar_catalogo(2_000, 1_000, 5, 11).unwrap();
    for cliente in c.ids_clientes().into_iter().step_by(50) {
        let recs = recomendar_para_cliente(&c, cliente, 10).unwrap();
        assert!(recs.len() <= 10);
        let unicos: HashSet<_> = recs.iter().map(|r| r.produto_id).collect();
        assert_eq!(unicos.len(), recs.len());
        for r in &recs {
            assert!(!c.ja_comprou(cliente, r.produto_id));
        }
        for par in recs.windows(2) {
            assert!(par[0].pontuacao >= par[1].pontuacao);
        }
    }
}

#[test]
fn recomendacao_por_produto_nao_devolve_o_proprio_produto() {
    let c = catalogo_exemplo().unwrap();
    let recs = recomendar_para_produto(&c, 11, 10).unwrap();
    assert!(!recs.is_empty());
    assert!(recs.iter().all(|r| r.produto_id != 11));
}
