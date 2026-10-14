//! Catálogo da MegaStore: cadastro e consulta de entidades, mais o grafo que
//! liga clientes, produtos e categorias.
//!
//! Estruturas:
//! - `HashMap<Id, Produto>` (e equivalentes para clientes e categorias): consulta
//!   por identificador em O(1) médio, que é a operação mais frequente.
//! - `HashMap<Id, HashSet<Id>>` com o histórico de compras de cada cliente:
//!   responde "o cliente já comprou este produto?" em O(1) médio, usado para
//!   excluir da recomendação o que ele já tem.
//! - `Grafo`: as relações. Arestas cliente–produto (compra, peso = avaliação) e
//!   produto–categoria (pertencimento, peso fixo menor).

use std::collections::{HashMap, HashSet};

use crate::avaliacao::{NOTA_PADRAO, nota_valida, peso_da_avaliacao};
use crate::grafo::Grafo;
use crate::modelo::{Categoria, Cliente, ErroCatalogo, Id, Produto, Vertice};

/// Peso da aresta produto–categoria. Menor que o de uma compra bem avaliada
/// (até 1.0), para que "quem comprou também comprou" pese mais que "é da mesma
/// categoria", que é justamente o que o sistema antigo fazia.
pub const PESO_CATEGORIA: f64 = 0.3;

#[derive(Debug, Default, Clone)]
pub struct Catalogo {
    categorias: HashMap<Id, Categoria>,
    produtos: HashMap<Id, Produto>,
    clientes: HashMap<Id, Cliente>,
    historico: HashMap<Id, HashSet<Id>>,
    grafo: Grafo,
}

impl Catalogo {
    pub fn new() -> Self {
        Self::default()
    }

    // ----- cadastro --------------------------------------------------------

    pub fn cadastrar_categoria(&mut self, id: Id, nome: &str) -> Result<(), ErroCatalogo> {
        if self.categorias.contains_key(&id) {
            return Err(ErroCatalogo::IdDuplicado(Vertice::Categoria(id)));
        }
        self.categorias.insert(
            id,
            Categoria {
                id,
                nome: nome.to_string(),
            },
        );
        self.grafo.adicionar_vertice(Vertice::Categoria(id));
        Ok(())
    }

    /// Cadastra o produto e o liga à sua categoria no grafo.
    pub fn cadastrar_produto(
        &mut self,
        id: Id,
        nome: &str,
        categoria_id: Id,
        preco: f64,
    ) -> Result<(), ErroCatalogo> {
        if self.produtos.contains_key(&id) {
            return Err(ErroCatalogo::IdDuplicado(Vertice::Produto(id)));
        }
        if !self.categorias.contains_key(&categoria_id) {
            return Err(ErroCatalogo::CategoriaNaoEncontrada(categoria_id));
        }
        self.produtos.insert(
            id,
            Produto {
                id,
                nome: nome.to_string(),
                categoria_id,
                preco,
            },
        );
        self.grafo.adicionar_aresta(
            Vertice::Produto(id),
            Vertice::Categoria(categoria_id),
            PESO_CATEGORIA,
        );
        Ok(())
    }

    pub fn cadastrar_cliente(&mut self, id: Id, nome: &str) -> Result<(), ErroCatalogo> {
        if self.clientes.contains_key(&id) {
            return Err(ErroCatalogo::IdDuplicado(Vertice::Cliente(id)));
        }
        self.clientes.insert(
            id,
            Cliente {
                id,
                nome: nome.to_string(),
            },
        );
        self.grafo.adicionar_vertice(Vertice::Cliente(id));
        Ok(())
    }

    /// Registra uma compra, com avaliação opcional (1 a 5).
    ///
    /// O peso da aresta é a avaliação convertida para 0.0–1.0; sem avaliação,
    /// usa a nota neutra. Recomprar soma peso à ligação existente.
    pub fn registrar_compra(
        &mut self,
        cliente_id: Id,
        produto_id: Id,
        nota: Option<u8>,
    ) -> Result<(), ErroCatalogo> {
        if !self.clientes.contains_key(&cliente_id) {
            return Err(ErroCatalogo::ClienteNaoEncontrado(cliente_id));
        }
        if !self.produtos.contains_key(&produto_id) {
            return Err(ErroCatalogo::ProdutoNaoEncontrado(produto_id));
        }
        let nota = nota.unwrap_or(NOTA_PADRAO);
        if !nota_valida(nota) {
            return Err(ErroCatalogo::NotaInvalida(nota));
        }
        self.grafo.adicionar_aresta(
            Vertice::Cliente(cliente_id),
            Vertice::Produto(produto_id),
            peso_da_avaliacao(nota),
        );
        self.historico
            .entry(cliente_id)
            .or_default()
            .insert(produto_id);
        Ok(())
    }

    // ----- consulta --------------------------------------------------------

    /// Consulta por identificador. O(1) médio.
    pub fn buscar_produto(&self, id: Id) -> Option<&Produto> {
        self.produtos.get(&id)
    }

    pub fn buscar_cliente(&self, id: Id) -> Option<&Cliente> {
        self.clientes.get(&id)
    }

    pub fn buscar_categoria(&self, id: Id) -> Option<&Categoria> {
        self.categorias.get(&id)
    }

    /// Busca por trecho do nome, sem diferenciar maiúsculas. Percorre todos os
    /// produtos: O(n). Para catálogos grandes, um índice invertido resolveria;
    /// fica como evolução (ver relatório).
    pub fn buscar_por_nome(&self, trecho: &str) -> Vec<&Produto> {
        let trecho = trecho.to_lowercase();
        let mut achados: Vec<&Produto> = self
            .produtos
            .values()
            .filter(|p| p.nome.to_lowercase().contains(&trecho))
            .collect();
        achados.sort_by_key(|p| p.id);
        achados
    }

    /// Todos os produtos, ordenados por id.
    pub fn listar_produtos(&self) -> Vec<&Produto> {
        let mut todos: Vec<&Produto> = self.produtos.values().collect();
        todos.sort_by_key(|p| p.id);
        todos
    }

    /// Ids dos produtos que o cliente já comprou (conjunto vazio se nenhum).
    pub fn historico(&self, cliente_id: Id) -> HashSet<Id> {
        self.historico.get(&cliente_id).cloned().unwrap_or_default()
    }

    pub fn ja_comprou(&self, cliente_id: Id, produto_id: Id) -> bool {
        self.historico
            .get(&cliente_id)
            .is_some_and(|compras| compras.contains(&produto_id))
    }

    pub fn grafo(&self) -> &Grafo {
        &self.grafo
    }

    pub fn num_produtos(&self) -> usize {
        self.produtos.len()
    }

    pub fn num_clientes(&self) -> usize {
        self.clientes.len()
    }

    pub fn num_categorias(&self) -> usize {
        self.categorias.len()
    }

    /// Ids de todos os clientes, ordenados.
    pub fn ids_clientes(&self) -> Vec<Id> {
        let mut ids: Vec<Id> = self.clientes.keys().copied().collect();
        ids.sort_unstable();
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Catalogo {
        let mut c = Catalogo::new();
        c.cadastrar_categoria(1, "Informática").unwrap();
        c.cadastrar_produto(10, "Mouse sem fio", 1, 89.9).unwrap();
        c.cadastrar_cliente(1, "Ana").unwrap();
        c
    }

    #[test]
    fn cadastra_e_consulta_produto() {
        let c = base();
        let p = c.buscar_produto(10).expect("produto deveria existir");
        assert_eq!(p.nome, "Mouse sem fio");
        assert_eq!(p.categoria_id, 1);
    }

    #[test]
    fn consulta_de_produto_inexistente_devolve_none() {
        let c = base();
        assert!(c.buscar_produto(999).is_none());
    }

    #[test]
    fn produto_com_categoria_inexistente_e_recusado() {
        let mut c = base();
        let r = c.cadastrar_produto(11, "Teclado", 42, 150.0);
        assert_eq!(r, Err(ErroCatalogo::CategoriaNaoEncontrada(42)));
    }

    #[test]
    fn id_duplicado_e_recusado() {
        let mut c = base();
        let r = c.cadastrar_produto(10, "Outro", 1, 1.0);
        assert_eq!(r, Err(ErroCatalogo::IdDuplicado(Vertice::Produto(10))));
    }

    #[test]
    fn produto_fica_ligado_a_categoria_no_grafo() {
        let c = base();
        let vizinhos = c.grafo().vizinhos(Vertice::Produto(10));
        assert_eq!(vizinhos.len(), 1);
        assert_eq!(vizinhos[0].destino, Vertice::Categoria(1));
    }

    #[test]
    fn compra_cria_aresta_e_historico() {
        let mut c = base();
        c.registrar_compra(1, 10, Some(5)).unwrap();
        assert!(c.ja_comprou(1, 10));
        let aresta = c
            .grafo()
            .vizinhos(Vertice::Cliente(1))
            .iter()
            .find(|a| a.destino == Vertice::Produto(10))
            .copied()
            .expect("aresta de compra");
        assert!((aresta.peso - 1.0).abs() < 1e-9);
    }

    #[test]
    fn compra_com_nota_invalida_e_recusada() {
        let mut c = base();
        assert_eq!(
            c.registrar_compra(1, 10, Some(9)),
            Err(ErroCatalogo::NotaInvalida(9))
        );
        assert!(!c.ja_comprou(1, 10));
    }

    #[test]
    fn compra_de_cliente_ou_produto_inexistente_e_recusada() {
        let mut c = base();
        assert_eq!(
            c.registrar_compra(7, 10, None),
            Err(ErroCatalogo::ClienteNaoEncontrado(7))
        );
        assert_eq!(
            c.registrar_compra(1, 77, None),
            Err(ErroCatalogo::ProdutoNaoEncontrado(77))
        );
    }

    #[test]
    fn busca_por_nome_ignora_maiusculas() {
        let c = base();
        assert_eq!(c.buscar_por_nome("MOUSE").len(), 1);
        assert!(c.buscar_por_nome("geladeira").is_empty());
    }
}
