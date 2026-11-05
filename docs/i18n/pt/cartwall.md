# Cartucheira

A cartucheira é o painel de botões por baixo dos leitores, ou ao lado deles
(ver [Disposição](layout.md#move-and-resize-the-cartwall)). Cada botão, um
**cartucho**, toca um som instantaneamente: jingles, efeitos, spots. Os
cartuchos tocam nas suas próprias saídas, independentemente dos leitores.

![A cartucheira com um cartucho a tocar](../../images/guide/cartwall.png)

## Utilização {#using-it}

- **Clique num cartucho** para o disparar. **Clique de novo** para o parar.
- **Arraste faixas da [janela da biblioteca](library.md)** para um cartucho
  para pôr nele a primeira (substitui o seu ficheiro quando está parado). Um
  cartucho que está a tocar ou no CUE recusa o largar.
- **Parar tudo** (extremo direito da barra) para todos os cartuchos que
  estão a tocar, em todas as páginas. A sua etiqueta mostra quantos estão a
  tocar, como em **Parar tudo (2)**; sem nenhum a tocar, fica esbatido e não
  mostra contagem.
- Enquanto um cartucho toca, a sua borda e uma barra que cresce a partir da
  esquerda à medida que toca assumem a [cor](#setting-up-carts) do cartucho
  (vermelho para um cartucho sem cor), uma linha fina marca a margem já
  tocada sobre a forma de onda e o seu tempo conta em decrescente. A linha
  move-se com suavidade, mesmo num cartucho curto.
- Depois de o ficheiro de um cartucho ser analisado, a sua forma de onda
  aparece ténue por trás do nome, para distinguir um som que ataca de
  imediato de um que entra aos poucos. Cobre o que o cartucho toca, do seu
  cue-in ao seu cue-out.
- Os cartuchos **sobrepõem-se** por predefinição: disparar um segundo não
  para o primeiro. Um cartucho configurado com **Parar os outros cartuchos ao
  disparar** para primeiro todos os outros cartuchos no ar, em qualquer
  página.
- Um cartucho configurado com **Em ciclo** recomeça no seu cue-in quando
  chega ao fim, sem interrupção, até o parar.
- **Clique com o botão direito** num cartucho para mais opções:

| Item | Ação |
|---|---|
| Pré-escutar no CUE | Tocá-lo na saída CUE da cartucheira (esbatido quando a cartucheira não tem saída Cue). Na saída que está no ar, pergunta primeiro enquanto algo toca aí (ver [CUE](players.md#cue-pre-listen)) |
| Parar | Pará-lo |
| Editar… | Abri-lo nas Definições |

O menu de um cartucho vazio só tem **Editar…**, para escolher o seu ficheiro.

- **Páginas:** os separadores ao lado de **CARTUCHEIRA** mudam de página. Um
  ponto vermelho indica que um cartucho dessa página está a tocar.
- Clique em **CARTUCHEIRA** para recolher a cartucheira ou voltar a
  expandi-la. Recolhida ao lado dos leitores, é uma faixa estreita com uma
  seta para a expandir e, enquanto um cartucho toca, um ponto vermelho.
- Por baixo dos leitores, com a altura que as suas linhas precisam, os
  botões encolhem quando a janela é baixa (até uma altura mínima) para que
  caibam todas as linhas configuradas. Depois de redimensionar a cartucheira,
  ou ao lado dos leitores, os botões crescem para a preencher, e os nomes e
  os tempos mantêm o tamanho no meio de cada botão. Em ambos os casos, a
  cartucheira só se desloca quando nem os botões mais pequenos cabem.

| Aspeto do botão | Significado |
|---|---|
| Ponto violeta | Jingle |
| Ponto âmbar | Efeito |
| Ponto cinzento | Spot (publicidade) |
| Borda, tom e ponto coloridos | A cor do cartucho, escolhida nas Definições: o ponto assume a cor em vez da do tipo |
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

O atalho que recolhe ou expande a cartucheira não faz nada enquanto a
cartucheira está oculta na barra superior (ver
[Disposição](layout.md#show-and-hide-panels)).

## Configurar os cartuchos {#setting-up-carts}

Vá a **Definições → Cartucheira**:

- **Páginas:** criar, mudar o nome, eliminar (a última página não pode ser
  eliminada) e definir o tamanho da grelha (linhas × colunas). Uma grelha
  mais pequena é recusada se eliminasse cartuchos que têm um ficheiro. Um
  tamanho escrito na caixa ao lado de um controlo deslizante aplica-se ao
  premir Enter ou ao sair da caixa; Escape cancela-o.
- **Importar… / Exportar…** guardam uma página num ficheiro
  `.cartpage.json` e carregam-na de novo, por exemplo para a partilhar entre
  estúdios. Os caminhos relativos dos ficheiros são resolvidos em relação à
  pasta do ficheiro. As cores viajam com a página; um ficheiro de uma versão
  anterior é importado sem elas.
- **Cartuchos:** clique num cartucho na grelha e defina o seu nome, ficheiro
  (**Escolher…** ou **Limpar**), tipo, **Cor**, **Em ciclo** e **Parar os
  outros cartuchos ao disparar**. **Cor** oferece sem cor e oito cores
  (vermelho, laranja, âmbar, verde, turquesa, azul, violeta, rosa): dê a
  todos os spots uma cor e aos indicativos da estação outra para os
  encontrar de relance. Um cartucho vazio mantém a sua cor, e limpar o
  ficheiro de um cartucho não a remove. Um cartucho com ficheiro tem também
  **Excluir da normalização** e **Ajuste**, as definições de
  [normalização do loudness](loudness.md#per-track-exclude-and-trim) da sua
  faixa, com a medição por baixo da etiqueta.

As saídas Main e Cue próprias da cartucheira estão em **Definições → Saídas
de áudio** (a linha **Cartucheira**).
