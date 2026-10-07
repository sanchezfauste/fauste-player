# Cartucheira

A cartucheira é a faixa de botões por baixo dos leitores. Cada botão, um
**cartucho**, toca um som instantaneamente: jingles, efeitos, spots. Os
cartuchos tocam nas suas próprias saídas, independentemente dos leitores.

![A cartucheira com um cartucho a tocar](../../images/guide/cartwall.png)

## Utilização {#using-it}

- **Clique num cartucho** para o disparar. **Clique de novo** para o parar.
- **Parar tudo** (extremo direito da barra) para todos os cartuchos que
  estão a tocar, em todas as páginas. A sua etiqueta mostra quantos estão a
  tocar, como em **Parar tudo (2)**; sem nenhum a tocar, fica esbatido e não
  mostra contagem.
- Enquanto um cartucho toca, a sua borda fica vermelha, uma barra vermelha
  encolhe à medida que toca e o seu tempo conta em decrescente.
- Os cartuchos **sobrepõem-se** por predefinição: disparar um segundo não
  para o primeiro. Um cartucho configurado com **Parar os outros cartuchos ao
  disparar** para primeiro todos os outros cartuchos no ar, em qualquer
  página.
- Um cartucho configurado com **Em ciclo** recomeça no seu cue-in quando
  chega ao fim, sem interrupção, até o parar.
- **Clique com o botão direito** num cartucho para mais opções:

| Item | Ação |
|---|---|
| Pré-escutar no CUE | Tocá-lo na saída CUE da cartucheira (esbatido quando a cartucheira não tem uma saída Cue à parte da sua saída Main) |
| Parar | Pará-lo |
| Editar… | Abri-lo nas Definições |

O menu de um cartucho vazio só tem **Editar…**, para escolher o seu ficheiro.

- **Páginas:** os separadores ao lado de **CARTUCHEIRA** mudam de página. Um
  ponto vermelho indica que um cartucho dessa página está a tocar.
- Clique em **CARTUCHEIRA** para recolher a faixa ou voltar a expandi-la.
- Quando a janela é baixa, os botões encolhem (até uma altura mínima) para
  que caibam todas as linhas configuradas; a cartucheira só se desloca
  quando nem os botões mais pequenos cabem.

| Aspeto do botão | Significado |
|---|---|
| Ponto violeta | Jingle |
| Ponto âmbar | Efeito |
| Ponto cinzento | Spot (publicidade) |
| ↻ depois do tipo | Em ciclo |
| ✋ depois do tipo | Para os outros cartuchos ao disparar |
| Ficheiro com uma cruz / sinal de aviso | O ficheiro não foi encontrado / não pode ser descodificado; passe o rato sobre o cartucho para ver o motivo e o caminho. Um ficheiro em falta é procurado de novo a cada 30 s (`tuning.missing_recheck_ms`). |
| «Vazio», esbatido | Sem ficheiro atribuído |

Os cartuchos usam os mesmos marcadores que as faixas. Começam no seu cue-in
e acabam no seu cue-out, que pode editar na forma de onda de um leitor quando
o ficheiro está carregado nele.

## Teclado {#keyboard}

Por predefinição, **F1**…**F12** disparam os cartuchos 1–12 da página
visível e **Ctrl+Space** para todos os cartuchos (o mesmo que **Parar
tudo**; também para um cartucho que esteja a pré-escutar no CUE, mesmo que
nenhum cartucho esteja a tocar). Consulte [Teclado](keyboard.md) para os
alterar.

## Configurar os cartuchos {#setting-up-carts}

Vá a **Definições → Cartucheira**:

- **Páginas:** criar, mudar o nome, eliminar (a última página não pode ser
  eliminada) e definir o tamanho da grelha (linhas × colunas). Uma grelha
  mais pequena é recusada se eliminasse cartuchos que têm um ficheiro.
- **Importar… / Exportar…** guardam uma página num ficheiro
  `.cartpage.json` e carregam-na de novo, por exemplo para a partilhar entre
  estúdios. Os caminhos relativos dos ficheiros são resolvidos em relação à
  pasta do ficheiro.
- **Cartuchos:** clique num cartucho na grelha e defina o seu nome, ficheiro
  (**Escolher…** ou **Limpar**), tipo, **Em ciclo** e **Parar os outros
  cartuchos ao disparar**.

As saídas Main e Cue próprias da cartucheira estão em **Definições → Saídas
de áudio** (a linha **Cartucheira**).
