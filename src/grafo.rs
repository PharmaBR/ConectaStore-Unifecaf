//! Grafo ponderado e não dirigido, armazenado como lista de adjacência.
//!
//! Estrutura: `HashMap<Vertice, Vec<Aresta>>`.
//! - `HashMap` dá acesso O(1) médio à lista de vizinhos de qualquer vértice.
//! - `Vec<Aresta>` guarda só as arestas que existem: memória O(V + E).
//!
//! Uma matriz de adjacência gastaria O(V²): com 1 milhão de vértices seriam 10¹²
//! células, quase todas vazias, porque cada cliente compra uma fração ínfima do
//! catálogo. Por isso a lista de adjacência é a escolha para este domínio.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::modelo::{Aresta, Vertice};

#[derive(Debug, Default, Clone)]
pub struct Grafo {
    adjacencia: HashMap<Vertice, Vec<Aresta>>,
    num_arestas: usize,
}

impl Grafo {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insere o vértice sem arestas. Não faz nada se ele já existir. O(1) médio.
    pub fn adicionar_vertice(&mut self, v: Vertice) {
        self.adjacencia.entry(v).or_default();
    }

    pub fn contem(&self, v: Vertice) -> bool {
        self.adjacencia.contains_key(&v)
    }

    /// Liga `a` e `b` nos dois sentidos (grafo não dirigido).
    ///
    /// Se a ligação já existe, o peso é somado em vez de criar uma aresta repetida:
    /// duas compras do mesmo produto pelo mesmo cliente reforçam a ligação.
    /// Custo: O(grau(a) + grau(b)) para procurar a aresta existente.
    pub fn adicionar_aresta(&mut self, a: Vertice, b: Vertice, peso: f64) {
        let nova = Self::somar_ou_inserir(self.adjacencia.entry(a).or_default(), b, peso);
        Self::somar_ou_inserir(self.adjacencia.entry(b).or_default(), a, peso);
        if nova {
            self.num_arestas += 1;
        }
    }

    /// Devolve `true` se a aresta é nova.
    fn somar_ou_inserir(lista: &mut Vec<Aresta>, destino: Vertice, peso: f64) -> bool {
        match lista.iter_mut().find(|aresta| aresta.destino == destino) {
            Some(existente) => {
                existente.peso += peso;
                false
            }
            None => {
                lista.push(Aresta { destino, peso });
                true
            }
        }
    }

    /// Vizinhos de `v`; fatia vazia se o vértice não existe. O(1) médio.
    pub fn vizinhos(&self, v: Vertice) -> &[Aresta] {
        match self.adjacencia.get(&v) {
            Some(lista) => lista,
            None => &[],
        }
    }

    pub fn grau(&self, v: Vertice) -> usize {
        self.vizinhos(v).len()
    }

    pub fn num_vertices(&self) -> usize {
        self.adjacencia.len()
    }

    /// Número de arestas não dirigidas (cada ligação conta uma vez).
    pub fn num_arestas(&self) -> usize {
        self.num_arestas
    }

    /// Busca em largura a partir de `origem`, até `profundidade_max` ligações.
    ///
    /// Devolve cada vértice alcançado com sua distância, na ordem de visita
    /// (do mais próximo para o mais distante).
    /// - `fila` (`VecDeque`): decide a ordem; quem entra primeiro sai primeiro.
    /// - `visitados` (`HashSet`): impede revisitar um vértice; é o que garante
    ///   que a busca termina mesmo com ciclos.
    ///
    /// Custo: O(V + E) do trecho do grafo alcançado.
    pub fn bfs(&self, origem: Vertice, profundidade_max: usize) -> Vec<(Vertice, usize)> {
        if !self.contem(origem) {
            return Vec::new();
        }

        let mut visitados = HashSet::new();
        let mut fila = VecDeque::new();
        let mut ordem = Vec::new();

        visitados.insert(origem);
        fila.push_back((origem, 0));

        while let Some((atual, distancia)) = fila.pop_front() {
            ordem.push((atual, distancia));
            if distancia == profundidade_max {
                continue;
            }
            for aresta in self.vizinhos(atual) {
                if visitados.insert(aresta.destino) {
                    fila.push_back((aresta.destino, distancia + 1));
                }
            }
        }
        ordem
    }

    /// Memória aproximada da lista de adjacência, em bytes: entradas do mapa
    /// mais as arestas armazenadas (cada ligação aparece nas duas pontas).
    pub fn memoria_estimada_bytes(&self) -> usize {
        let por_entrada = std::mem::size_of::<Vertice>() + std::mem::size_of::<Vec<Aresta>>();
        let por_aresta = std::mem::size_of::<Aresta>();
        self.num_vertices() * por_entrada + 2 * self.num_arestas * por_aresta
    }

    /// Memória que uma matriz de adjacência de pesos `f64` ocuparia, em bytes.
    pub fn memoria_matriz_bytes(&self) -> u128 {
        let v = self.num_vertices() as u128;
        v * v * std::mem::size_of::<f64>() as u128
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ANA: Vertice = Vertice::Cliente(1);
    const BIA: Vertice = Vertice::Cliente(2);
    const MOUSE: Vertice = Vertice::Produto(10);
    const TECLADO: Vertice = Vertice::Produto(11);

    #[test]
    fn grafo_novo_esta_vazio() {
        let g = Grafo::new();
        assert_eq!(g.num_vertices(), 0);
        assert_eq!(g.num_arestas(), 0);
        assert!(g.vizinhos(ANA).is_empty());
    }

    #[test]
    fn aresta_liga_os_dois_lados() {
        let mut g = Grafo::new();
        g.adicionar_aresta(ANA, MOUSE, 1.0);
        assert_eq!(g.vizinhos(ANA)[0].destino, MOUSE);
        assert_eq!(g.vizinhos(MOUSE)[0].destino, ANA);
        assert_eq!(g.num_vertices(), 2);
        assert_eq!(g.num_arestas(), 1);
    }

    #[test]
    fn aresta_repetida_soma_peso_em_vez_de_duplicar() {
        let mut g = Grafo::new();
        g.adicionar_aresta(ANA, MOUSE, 0.6);
        g.adicionar_aresta(ANA, MOUSE, 0.4);
        assert_eq!(g.grau(ANA), 1);
        assert_eq!(g.num_arestas(), 1);
        assert!((g.vizinhos(ANA)[0].peso - 1.0).abs() < 1e-9);
        assert!((g.vizinhos(MOUSE)[0].peso - 1.0).abs() < 1e-9);
    }

    #[test]
    fn adicionar_vertice_existente_nao_apaga_arestas() {
        let mut g = Grafo::new();
        g.adicionar_aresta(ANA, MOUSE, 1.0);
        g.adicionar_vertice(ANA);
        assert_eq!(g.grau(ANA), 1);
    }

    #[test]
    fn bfs_em_vertice_inexistente_devolve_vazio() {
        let g = Grafo::new();
        assert!(g.bfs(ANA, 3).is_empty());
    }

    #[test]
    fn bfs_em_vertice_isolado_devolve_so_ele() {
        let mut g = Grafo::new();
        g.adicionar_vertice(ANA);
        assert_eq!(g.bfs(ANA, 3), vec![(ANA, 0)]);
    }

    #[test]
    fn bfs_termina_em_grafo_com_ciclo() {
        // Ana - Mouse - Bia - Teclado - Ana forma um ciclo.
        let mut g = Grafo::new();
        g.adicionar_aresta(ANA, MOUSE, 1.0);
        g.adicionar_aresta(MOUSE, BIA, 1.0);
        g.adicionar_aresta(BIA, TECLADO, 1.0);
        g.adicionar_aresta(TECLADO, ANA, 1.0);
        let ordem = g.bfs(ANA, 10);
        assert_eq!(ordem.len(), 4);
        assert_eq!(ordem[0], (ANA, 0));
        assert_eq!(ordem[3], (BIA, 2));
    }

    #[test]
    fn bfs_respeita_profundidade_maxima() {
        let mut g = Grafo::new();
        g.adicionar_aresta(ANA, MOUSE, 1.0);
        g.adicionar_aresta(MOUSE, BIA, 1.0);
        g.adicionar_aresta(BIA, TECLADO, 1.0);
        let ordem = g.bfs(ANA, 2);
        assert_eq!(ordem, vec![(ANA, 0), (MOUSE, 1), (BIA, 2)]);
    }

    #[test]
    fn matriz_gasta_mais_que_lista_em_grafo_esparso() {
        let mut g = Grafo::new();
        for i in 0..1000 {
            g.adicionar_aresta(Vertice::Cliente(i), Vertice::Produto(i % 50), 1.0);
        }
        assert!(g.memoria_matriz_bytes() > 10 * g.memoria_estimada_bytes() as u128);
    }
}
