# ConectaStore — Sistema de Recomendação de Produtos Baseado em Grafos

Trabalho da disciplina **Data Structure Strategy and Implementation** (UniFECAF).
Autor: Breno Silva de Abreu.

**Vídeo pitch:** [LINK DO VÍDEO AQUI](https://youtu.be/)

---

## 1. Objetivo e funcionamento

A MegaStore vende milhões de produtos, mas recomenda apenas os mais vendidos ou
itens da mesma categoria. O ConectaStore usa as **relações** entre clientes,
compras, avaliações e categorias para recomendar o que é relevante para cada
cliente.

O catálogo é modelado como um **grafo**:

- **Vértices:** clientes, produtos e categorias.
- **Arestas:**
  - *compra* (cliente — produto), com peso igual à avaliação da compra
    convertida para 0,2–1,0 (nota 1 a 5; sem avaliação, nota neutra 3);
  - *pertencimento* (produto — categoria), com peso fixo 0,3.
- **Tipo:** não dirigido e ponderado. Na prática é tripartido: não há arestas
  entre dois clientes nem entre dois produtos.

Para recomendar, o sistema faz uma **busca em largura (BFS)** a partir do cliente,
limitada a 3 ligações:

```text
cliente → produto comprado → outro cliente que comprou o mesmo → produto dele
cliente → produto comprado → categoria do produto → outro produto da categoria
```

Cada produto alcançado acumula uma pontuação: a soma, sobre todos os caminhos
mais curtos até ele, do produto dos pesos das arestas. Produtos já comprados são
excluídos, e o resultado sai ordenado da maior para a menor pontuação. Também é
possível recomendar a partir de um produto ("quem comprou isto também comprou").

## 2. Tecnologias e estruturas utilizadas

- **Rust** (edição 2024), apenas a biblioteca padrão, sem dependências externas.
- **Cargo** para compilação, execução e testes.

| Estrutura | Onde | Por quê |
|---|---|---|
| `HashMap<Vertice, Vec<Aresta>>` | lista de adjacência do grafo | vizinhos de um vértice em O(1) médio; memória O(V + E) |
| `HashMap<Id, Produto>` (e clientes, categorias) | cadastro | consulta por id em O(1) médio |
| `HashMap<Id, HashSet<Id>>` | histórico de compras | "já comprou?" em O(1) médio |
| `VecDeque<Vertice>` | fila da BFS | inserir no fim e retirar do início em O(1) |
| `HashMap<Vertice, usize>` | distâncias da BFS (visitados) | garante término e corte por profundidade |
| `HashMap<Vertice, f64>` | pontuação acumulada | uma entrada por produto: impede recomendação duplicada |
| `HashSet<Id>` | produtos a excluir | regra de negócio separada do controle da busca |
| `Vec<Recomendacao>` | resultado | ordenação por pontuação em O(c log c) |
| `enum Vertice` | tipo do vértice | cliente 1 e produto 1 são vértices diferentes |
| `Result<_, ErroCatalogo>` / `Option` | erros e ausências | o compilador obriga o tratamento; nenhum `unwrap()` fora dos testes |

Uma matriz de adjacência foi descartada: com 1,5 milhão de vértices ela ocuparia
cerca de 16,5 TB, contra cerca de 214 MB da lista (ver resultados).

## 3. Compilação e execução

Pré-requisito: Rust instalado via [rustup](https://rustup.rs).

```bash
git clone <URL-DO-REPOSITÓRIO>
cd megastore
cargo build --release
cargo run --release -- demo
```

Comandos disponíveis:

```text
cargo run -- demo                                   demonstração completa
cargo run -- produtos                               lista o catálogo de exemplo
cargo run -- produto <id>                           consulta um produto
cargo run -- buscar <texto>                         busca produtos pelo nome
cargo run -- cadastrar <id> <nome> <categoria> <preço>
cargo run -- recomendar-cliente <id> [limite]
cargo run -- recomendar-produto <id> [limite]
cargo run --release -- medir                        medição de desempenho
```

## 4. Testes

```bash
cargo test
```

São 46 testes:

- **39 unitários** (em cada módulo de `src/`): grafo vazio, vértice isolado,
  grafo com ciclo, profundidade máxima, aresta repetida, cadastro e consulta,
  erros de domínio, exclusão do histórico, duplicatas, ordenação e limite.
- **7 de integração** (`tests/`): fluxo completo pela API pública, regras de
  recomendação no catálogo de exemplo e em um catálogo gerado com 2.000 clientes.

## 5. Exemplos de uso

```text
$ cargo run -- recomendar-cliente 1 3
Recomendações para Ana (cliente 1):
  1. Teclado mecânico             pontuação 1.980
  2. Café especial 500g           pontuação 1.640
  3. Cadeira ergonômica           pontuação 1.600
```

Ana comprou Notebook, Mouse e um livro de Algoritmos. O Teclado lidera porque
dois clientes que compraram o Mouse (Bia e Helena) também compraram o Teclado.

```text
$ cargo run -- recomendar-produto 31 3
Quem comprou "Cadeira ergonômica" também comprou:
  1. Luminária de mesa            pontuação 1.090
  2. Notebook                     pontuação 0.800
  3. Mouse sem fio                pontuação 0.800

$ cargo run -- produto 12
Produto 12: Teclado mecânico | categoria: Informática | R$ 349.90

$ cargo run -- buscar livro
[20] Livro: Algoritmos - R$ 189.00
[21] Livro: Programando em Rust - R$ 159.00
[22] Livro: Estruturas de Dados - R$ 139.00

$ cargo run -- cadastrar 15 "Webcam HD" 1 249.90
Produto cadastrado.
Produto 15: Webcam HD | categoria: Informática | R$ 249.90

$ cargo run -- produto 99
erro: produto 99 não encontrado
```

## 6. Arquitetura da solução

```text
megastore/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs           interface de linha de comando
│   ├── lib.rs            raiz da biblioteca
│   ├── modelo.rs         Produto, Cliente, Categoria, Vertice, Aresta, ErroCatalogo
│   ├── avaliacao.rs      notas de 1 a 5 → peso da aresta
│   ├── grafo.rs          lista de adjacência + BFS
│   ├── catalogo.rs       cadastro e consulta; mantém grafo e histórico
│   ├── recomendacao.rs   algoritmo de recomendação
│   ├── dados.rs          catálogo de exemplo + gerador sintético
│   └── medicao.rs        medição de desempenho
└── tests/
    ├── integracao.rs
    └── avaliacao.rs
```

Fluxo: `main.rs` → `Catalogo` (cadastro e consulta) → `Grafo` (relações) →
`recomendacao` (BFS sobre o grafo, com o histórico do catálogo como exclusão).

O grafo usa **identificadores numéricos** (`u32`) em vez de ponteiros entre nós:
cada vértice é uma chave do `HashMap`, o que dispensa contagem de referências e
mantém o código simples e seguro.

## 7. Resultados dos testes de desempenho

Comando: `cargo run --release -- medir`. Cada cliente faz 5 compras; há uma
categoria a cada 100 produtos. Recomendação: média de 1.000 clientes sorteados
(10 resultados cada). Consulta por id: média de 100.000 consultas.

| Clientes | Produtos | Vértices | Arestas | Construção | Recomendação (média) | Consulta por id (média) | Busca por nome | Memória lista | Memória matriz |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1.000 | 500 | 1.505 | 5.480 | 3.3 ms | 118.0 µs | 18 ns | 0.03 ms | 218 KB | 17.3 MB |
| 10.000 | 5.000 | 15.050 | 54.984 | 28.4 ms | 164.6 µs | 21 ns | 0.62 ms | 2.1 MB | 1.7 GB |
| 100.000 | 50.000 | 150.500 | 549.983 | 469.2 ms | 204.6 µs | 62 ns | 9.02 ms | 21.4 MB | 168.8 GB |
| 1.000.000 | 500.000 | 1.505.000 | 5.499.980 | 7937.7 ms | 195.4 µs | 143 ns | 113.63 ms | 213.8 MB | 16.5 TB |

*Ambiente: Linux x86_64, 2 vCPUs. "Memória lista" é a estimativa do grafo
(entradas do mapa + arestas); "Memória matriz" é o que uma matriz de pesos `f64`
ocuparia.*

Leitura:

- **Recomendação** fica entre 0,1 e 0,2 ms com o catálogo 1.000 vezes maior: a
  BFS com profundidade 3 percorre só a vizinhança do cliente, cujo tamanho
  depende do grau médio, não do total de vértices (cerca de 750 vértices
  visitados em média). O pequeno aumento se deve, provavelmente, a falhas de
  cache em estruturas maiores.
- **Consulta por id** (`HashMap`) fica na casa de dezenas de nanossegundos em
  todos os volumes: O(1) médio.
- **Busca por nome** (varredura linear) cresce na proporção do catálogo: O(n).
- **Memória**: a lista cresce linearmente; a matriz, quadraticamente.

## 8. Vídeo pitch

[LINK DO VÍDEO AQUI](https://youtu.be/)
