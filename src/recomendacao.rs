//! Algoritmo de recomendação: busca em largura (BFS) com profundidade limitada
//! e pontuação acumulada por caminho.
//!
//! Ideia: produtos "próximos" de um cliente no grafo são bons candidatos.
//!
//! ```text
//! cliente --compra--> produto --compra--> outro cliente --compra--> PRODUTO RECOMENDADO
//!   (0)                 (1)                   (2)                      (3)
//! cliente --compra--> produto --pertence--> categoria --pertence--> PRODUTO RECOMENDADO
//! ```
//!
//! - Distância 1 a partir do cliente: o que ele já comprou (excluído).
//! - Distância 3: o que clientes parecidos compraram e produtos da mesma
//!   categoria do que ele comprou. É aí que estão as recomendações.
//! - Distância 5 ou mais: relação fraca e custo de busca muito maior; fica fora.
//!
//! Por que BFS e não DFS: a BFS visita por camadas de distância, então "mais
//! perto" é visitado antes de "mais longe", e o corte por profundidade é exato.
//! A DFS mergulharia num caminho até o fim antes de olhar os vizinhos diretos.
//!
//! Pontuação: cada vértice recebe uma "força" igual à soma, sobre todos os
//! caminhos mais curtos que chegam a ele, do produto dos pesos das arestas.
//! Um produto alcançado por vários clientes parecidos acumula mais força.
//!
//! Três estruturas separadas, de propósito:
//! - `distancia` (`HashMap<Vertice, usize>`): também é o conjunto de visitados.
//!   Controla a terminação da busca.
//! - `excluir` (`HashSet<Id>`): regra de negócio (histórico do cliente). Se o
//!   histórico fosse marcado como visitado, a busca não passaria por ele, e é
//!   justamente pelo histórico que passam os melhores caminhos.
//! - `forca` (`HashMap<Vertice, f64>`): uma entrada por vértice, então um
//!   produto alcançado por dois caminhos soma pontos em vez de aparecer duas
//!   vezes. É o que impede recomendações duplicadas.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::catalogo::Catalogo;
use crate::grafo::Grafo;
use crate::modelo::{ErroCatalogo, Id, Vertice};

/// Profundidade a partir de um cliente: cliente → produto → cliente → produto.
pub const PROFUNDIDADE_CLIENTE: usize = 3;
/// Profundidade a partir de um produto: produto → cliente → produto.
pub const PROFUNDIDADE_PRODUTO: usize = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct Recomendacao {
    pub produto_id: Id,
    pub pontuacao: f64,
}

/// Recomenda até `limite` produtos para o cliente, excluindo o que ele já comprou.
pub fn recomendar_para_cliente(
    catalogo: &Catalogo,
    cliente_id: Id,
    limite: usize,
) -> Result<Vec<Recomendacao>, ErroCatalogo> {
    if catalogo.buscar_cliente(cliente_id).is_none() {
        return Err(ErroCatalogo::ClienteNaoEncontrado(cliente_id));
    }
    let excluir = catalogo.historico(cliente_id);
    Ok(recomendar_a_partir(
        catalogo.grafo(),
        Vertice::Cliente(cliente_id),
        PROFUNDIDADE_CLIENTE,
        &excluir,
        limite,
    ))
}

/// "Quem comprou este produto também comprou": recomenda a partir de um produto.
pub fn recomendar_para_produto(
    catalogo: &Catalogo,
    produto_id: Id,
    limite: usize,
) -> Result<Vec<Recomendacao>, ErroCatalogo> {
    if catalogo.buscar_produto(produto_id).is_none() {
        return Err(ErroCatalogo::ProdutoNaoEncontrado(produto_id));
    }
    let excluir = HashSet::from([produto_id]);
    Ok(recomendar_a_partir(
        catalogo.grafo(),
        Vertice::Produto(produto_id),
        PROFUNDIDADE_PRODUTO,
        &excluir,
        limite,
    ))
}

/// Núcleo do algoritmo: BFS por camadas acumulando a força de cada vértice.
///
/// Custo: O(V' + E') para percorrer o trecho alcançado até a profundidade
/// máxima, mais O(c log c) para ordenar os c candidatos. V' e E' dependem do
/// grau médio e da profundidade, não do tamanho total do catálogo.
pub fn recomendar_a_partir(
    grafo: &Grafo,
    origem: Vertice,
    profundidade_max: usize,
    excluir: &HashSet<Id>,
    limite: usize,
) -> Vec<Recomendacao> {
    let mut distancia: HashMap<Vertice, usize> = HashMap::new();
    let mut forca: HashMap<Vertice, f64> = HashMap::new();
    let mut fila: VecDeque<Vertice> = VecDeque::new();

    distancia.insert(origem, 0);
    forca.insert(origem, 1.0);
    fila.push_back(origem);

    while let Some(atual) = fila.pop_front() {
        let d = distancia[&atual];
        if d == profundidade_max {
            continue;
        }
        let forca_atual = forca[&atual];

        for aresta in grafo.vizinhos(atual) {
            let contribuicao = forca_atual * aresta.peso;
            match distancia.get(&aresta.destino) {
                // Primeira vez: entra na fila e na camada seguinte.
                None => {
                    distancia.insert(aresta.destino, d + 1);
                    forca.insert(aresta.destino, contribuicao);
                    fila.push_back(aresta.destino);
                }
                // Outro caminho mais curto até o mesmo vértice: soma força,
                // mas não entra na fila de novo.
                Some(&dd) if dd == d + 1 => {
                    *forca.entry(aresta.destino).or_insert(0.0) += contribuicao;
                }
                // Já alcançado por um caminho menor: ignora (evita voltar).
                Some(_) => {}
            }
        }
    }

    let mut candidatos: Vec<Recomendacao> = forca
        .into_iter()
        .filter_map(|(vertice, pontuacao)| match vertice {
            Vertice::Produto(id) if vertice != origem && !excluir.contains(&id) => {
                Some(Recomendacao {
                    produto_id: id,
                    pontuacao,
                })
            }
            _ => None,
        })
        .collect();

    // Maior pontuação primeiro; empate decidido pelo menor id (resultado estável).
    candidatos.sort_by(|a, b| {
        b.pontuacao
            .total_cmp(&a.pontuacao)
            .then(a.produto_id.cmp(&b.produto_id))
    });
    candidatos.truncate(limite);
    candidatos
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O exemplo da Ana, Bia e Caio:
    /// Ana comprou Notebook e Mouse; Bia, Mouse e Teclado; Caio, Teclado e Monitor.
    /// Notebook, Mouse, Teclado e Monitor ficam em categorias diferentes.
    fn exemplo() -> Catalogo {
        let mut c = Catalogo::new();
        for (id, nome) in [
            (1, "Notebooks"),
            (2, "Mouses"),
            (3, "Teclados"),
            (4, "Monitores"),
        ] {
            c.cadastrar_categoria(id, nome).unwrap();
        }
        c.cadastrar_produto(10, "Notebook", 1, 4000.0).unwrap();
        c.cadastrar_produto(11, "Mouse", 2, 90.0).unwrap();
        c.cadastrar_produto(12, "Teclado", 3, 150.0).unwrap();
        c.cadastrar_produto(13, "Monitor", 4, 900.0).unwrap();
        for (id, nome) in [(1, "Ana"), (2, "Bia"), (3, "Caio")] {
            c.cadastrar_cliente(id, nome).unwrap();
        }
        c.registrar_compra(1, 10, Some(5)).unwrap();
        c.registrar_compra(1, 11, Some(5)).unwrap();
        c.registrar_compra(2, 11, Some(5)).unwrap();
        c.registrar_compra(2, 12, Some(5)).unwrap();
        c.registrar_compra(3, 12, Some(5)).unwrap();
        c.registrar_compra(3, 13, Some(5)).unwrap();
        c
    }

    fn ids(recs: &[Recomendacao]) -> Vec<Id> {
        recs.iter().map(|r| r.produto_id).collect()
    }

    #[test]
    fn ana_recebe_teclado_e_nao_monitor() {
        let c = exemplo();
        let recs = recomendar_para_cliente(&c, 1, 10).unwrap();
        assert_eq!(ids(&recs), vec![12]); // Monitor está a 5 ligações
    }

    #[test]
    fn nunca_recomenda_o_que_o_cliente_ja_comprou() {
        let c = exemplo();
        let recs = recomendar_para_cliente(&c, 2, 10).unwrap();
        assert!(!ids(&recs).contains(&11));
        assert!(!ids(&recs).contains(&12));
        assert_eq!(ids(&recs), vec![10, 13]);
    }

    #[test]
    fn produto_alcancado_por_dois_caminhos_aparece_uma_vez_com_mais_pontos() {
        let mut c = exemplo();
        // Davi também comprou Mouse e Teclado: agora há dois caminhos Ana → Teclado.
        c.cadastrar_cliente(4, "Davi").unwrap();
        c.registrar_compra(4, 11, Some(5)).unwrap();
        c.registrar_compra(4, 12, Some(5)).unwrap();
        let antes = recomendar_para_cliente(&exemplo(), 1, 10).unwrap();
        let depois = recomendar_para_cliente(&c, 1, 10).unwrap();
        assert_eq!(ids(&depois), vec![12]);
        assert!(depois[0].pontuacao > antes[0].pontuacao);
    }

    #[test]
    fn compra_mais_frequente_pesa_mais_que_mesma_categoria() {
        let mut c = exemplo();
        // Mousepad é da mesma categoria do Mouse, mas ninguém o comprou.
        c.cadastrar_produto(14, "Mousepad", 2, 30.0).unwrap();
        let recs = recomendar_para_cliente(&c, 1, 10).unwrap();
        assert_eq!(ids(&recs), vec![12, 14]);
    }

    #[test]
    fn limite_corta_a_lista() {
        let c = exemplo();
        let recs = recomendar_para_cliente(&c, 2, 1).unwrap();
        assert_eq!(recs.len(), 1);
    }

    #[test]
    fn cliente_sem_compras_recebe_lista_vazia() {
        let mut c = exemplo();
        c.cadastrar_cliente(9, "Novo").unwrap();
        assert!(recomendar_para_cliente(&c, 9, 10).unwrap().is_empty());
    }

    #[test]
    fn cliente_inexistente_e_erro() {
        let c = exemplo();
        assert_eq!(
            recomendar_para_cliente(&c, 99, 10),
            Err(ErroCatalogo::ClienteNaoEncontrado(99))
        );
    }

    #[test]
    fn quem_comprou_mouse_tambem_comprou() {
        let c = exemplo();
        let recs = recomendar_para_produto(&c, 11, 10).unwrap();
        // Mouse → Ana → Notebook e Mouse → Bia → Teclado.
        let mut obtidos = ids(&recs);
        obtidos.sort_unstable();
        assert_eq!(obtidos, vec![10, 12]);
    }

    #[test]
    fn produto_inexistente_e_erro() {
        let c = exemplo();
        assert_eq!(
            recomendar_para_produto(&c, 99, 10),
            Err(ErroCatalogo::ProdutoNaoEncontrado(99))
        );
    }

    #[test]
    fn pontuacoes_saem_em_ordem_decrescente() {
        let c = crate::dados::catalogo_exemplo().unwrap();
        for cliente in c.ids_clientes() {
            let recs = recomendar_para_cliente(&c, cliente, 10).unwrap();
            for par in recs.windows(2) {
                assert!(par[0].pontuacao >= par[1].pontuacao);
            }
        }
    }
}
