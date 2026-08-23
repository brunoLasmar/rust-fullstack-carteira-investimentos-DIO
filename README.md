# Carteira de Investimentos em Rust

Aplicação web full-stack para cadastro de usuários, autenticação, gerenciamento de ativos e registro de compras de investimentos. O sistema calcula o valor investido, o valor atual da carteira e o retorno total de cada usuário.

Este projeto foi desenvolvido a partir do projeto base da Digital Innovation One, como uma resolução para o desafio proposto pelo bootcamp de Rust em conjunto com a Santander Open Academy:

<https://github.com/digitalinnovationone/rust-fullstack-carteira-investimentos.git>

## O que o projeto faz

A aplicação oferece:

- Cadastro de usuários.
- Login com autenticação por JWT armazenado em cookie.
- Logout.
- Listagem de ativos disponíveis.
- Cadastro e atualização de ativos pela API administrativa.
- Registro de compras de ativos pelo usuário.
- Exibição da quantidade, valor unitário e variação de cada ativo.
- Histórico de compras por ativo.
- Resumo geral da carteira com:
  - valor investido;
  - valor atual;
  - retorno total;
  - percentual de retorno.
- Mensagens de erro renderizadas na interface quando possível.
- Validação de quantidade positiva no backend e no banco de dados.

## Tecnologias utilizadas

### Backend

- Rust 2024 edition.
- Axum para HTTP e roteamento.
- Tokio para execução assíncrona.
- SQLx para acesso tipado ao PostgreSQL.
- Askama para renderização dos templates HTML.
- Serde para serialização e desserialização.
- `jwt-simple` para criação e validação de JWT.
- `password-auth` para geração e verificação de hashes de senha.
- `thiserror` para definição dos erros da aplicação.
- `tracing` e `tracing-subscriber` para logs.
- `dotenvy` para carregar variáveis de ambiente.

### Banco e frontend

- PostgreSQL.
- SQL para migrations e fixtures.
- HTML com templates Askama.
- Tailwind CSS via CDN.
- Fonte Space Mono via Google Fonts.

## Como executar

### Pré-requisitos

Instale:

- Rust e Cargo.
- Docker e Docker Compose.
- SQLx CLI, caso queira executar as migrations pelo terminal.

Para instalar o SQLx CLI:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

### 1. Configurar o banco

O projeto utiliza as variáveis definidas em `.env`:

```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
```

Inicie o PostgreSQL:

```bash
docker compose up -d db
```

### 2. Executar as migrations

A aplicação não executa migrations automaticamente ao iniciar. Execute-as com:

```bash
sqlx migrate run
```

As migrations criam as tabelas `users`, `assets` e `owned_assets`, além da restrição que impede quantidades menores ou iguais a zero.

### 3. Inserir ativos de teste

As fixtures ficam em `src/routes/fixtures/`. Elas são usadas automaticamente pelos testes SQLx, mas podem ser executadas manualmente no banco:

```bash
psql -h localhost -U postgres -d postgres \
  -f src/routes/fixtures/bitcoin_asset.sql
```

Outros exemplos disponíveis:

```bash
psql -h localhost -U postgres -d postgres \
  -f src/routes/fixtures/dolar_asset.sql

psql -h localhost -U postgres -d postgres \
  -f src/routes/fixtures/ethereum_asset.sql

psql -h localhost -U postgres -d postgres \
  -f src/routes/fixtures/real_asset.sql
```

A senha local configurada no `compose.yml` é `postgres`.

### 4. Iniciar a aplicação

```bash
cargo run
```

A aplicação ficará disponível em:

<http://localhost:3000>

## Como usar

1. Acesse `/register` e crie um usuário.
2. Acesse `/login` e faça login.
3. Abra `/assets`.
4. Clique em `register purchase`.
5. Selecione um ativo, informe o valor unitário e uma quantidade maior que zero.
6. Salve a compra.
7. Consulte o resumo da carteira e clique em um ativo para abrir seu histórico.

O valor atual de cada ativo é obtido de `assets.unit_value`. O retorno é calculado comparando esse valor com o preço registrado no momento da compra.

## Rotas principais

### Interface web

| Método | Rota | Função |
| --- | --- | --- |
| GET | `/` | Redireciona para assets ou login |
| GET/POST | `/login` | Exibe e processa o login |
| GET/POST | `/register` | Exibe e processa o cadastro |
| GET | `/logout` | Remove o cookie de autenticação |
| GET | `/assets` | Exibe a carteira do usuário |
| POST | `/assets` | Registra uma compra |

### API de assets

| Método | Rota | Função |
| --- | --- | --- |
| GET | `/api/assets` | Lista os ativos |
| POST | `/api/assets` | Cria um ativo com autorização administrativa |
| PATCH | `/api/assets` | Atualiza um ativo com autorização administrativa |

O corpo para criação de um ativo é:

```json
{
  "name": "Bitcoin",
  "unit_value": 60000.0
}
```

## Melhorias aplicadas sob o projeto base

### Nova tela de assets

Foi criada a tela `templates/assets.html`, que permite ao usuário:

- visualizar seus ativos;
- registrar compras;
- abrir e fechar o histórico de compras;
- consultar o resumo geral da carteira;
- visualizar mensagens de erro na própria página.

### Tratamento de erros na interface

Erros de login e de cadastro passaram a ser apresentados nas páginas correspondentes. O cadastro com username já existente, por exemplo, não retorna apenas uma resposta genérica.

### Proteção contra exposição de erros internos

Erros de banco, template e JWT não são enviados diretamente ao usuário. Os detalhes técnicos são registrados no log do servidor e o cliente recebe uma mensagem genérica.

### Status correto para falhas de autenticação

Tokens inválidos ou expirados retornam `401 Unauthorized`, em vez de `500 Internal Server Error`. A mensagem pública continua genérica para não revelar detalhes do token.

### Resumo financeiro da carteira

Foi criada a struct `PortfolioSummary` e uma consulta agregada que calcula:

```text
valor investido = soma(preço de compra x quantidade)
valor atual = soma(preço atual x quantidade)
retorno total = valor atual - valor investido
percentual de retorno = retorno total / valor investido x 100
```

O percentual é formatado com duas casas decimais na apresentação.

### Validação de quantidade positiva

A interface impede o valor zero e valores negativos. O backend também rejeita valores não positivos, `NaN` e infinito. O banco reforça a regra com:

```sql
CHECK (quantity_owned > 0)
```

### Fixtures adicionais

Foram adicionadas fixtures para ativos como Dolar, Ethereum e Real, facilitando testes e demonstrações locais.

### Suporte a datas no JSON

A dependência `time` foi configurada com a feature `serde`, permitindo serializar e desserializar `OffsetDateTime` no histórico de compras.

## Como testar esta versão

### Testes automatizados

Com o PostgreSQL disponível e `DATABASE_URL` configurada:

```bash
cargo test
```

Os testes que usam `#[sqlx::test]` criam bancos temporários e aplicam as migrations e fixtures necessárias.

### Teste manual da interface

1. Inicie o banco, aplique as migrations e insira pelo menos um ativo.
2. Execute `cargo run`.
3. Crie um usuário e faça login.
4. Registre uma compra com quantidade `1`.
5. Verifique o resumo da carteira.
6. Registre uma segunda compra e confira o histórico.
7. Tente informar quantidade `0` ou negativa. O cadastro deve ser recusado e a mensagem aparecer em vermelho.
8. Tente selecionar um asset inexistente pelo DevTools adicionando uma opção com valor `999`. O banco deve rejeitar a compra e a interface deve mostrar apenas uma mensagem genérica de erro.

### Teste da API administrativa

A API de criação e atualização exige o valor administrativo configurado no código atual. Em ambiente local, um exemplo é:

```bash
curl -X POST http://localhost:3000/api/assets \
  -H 'Authorization: im-the-admin' \
  -H 'Content-Type: application/json' \
  -d '{"name":"Bitcoin","unit_value":60000.0}'
```

A API de listagem não exige autenticação:

```bash
curl http://localhost:3000/api/assets
```

## O que aprendi com o desafio

De certa forma, aprender Rust reforçou praticamente todos os aprendizados que obtive na minha graduação em Ciência da Computação em relação à segurança, paradigmas de programação, designs arquiteturais e a necessidade de fundamentar uma base sólida para se construir uma aplicação.

Com as lutas contra o compilador, sinto que pude aprender diretamente o peso que algumas decisões têm sobre o funcionamento de uma função ou de uma struct. Com o tempo, fui percebendo que apesar de ter uma curva de aprendizado bem íngreme, aprender Rust fortifica as suas bases de conhecimento sobre programação e te força a manter padrões no código que em outras linguagens poderiam ser ignorados.

Esse foi meu primeiro bootcamp e certamente essa não será minha última aplicação em Rust.