# Cartucheira

A cartucheira é o panel de botóns baixo os reprodutores, ou ao seu carón
(ver [Disposición](layout.md#move-and-resize-the-cartwall)). Cada botón, un
**cartucho**, reproduce un son ao instante: jingles, efectos, cuñas. Os
cartuchos soan nas súas propias saídas, con independencia dos reprodutores.

![A cartucheira cun cartucho en reprodución](../../images/guide/cartwall.png)

## Uso {#using-it}

- **Fai clic nun cartucho** para disparalo. **Fai clic nel de novo** para
  paralo.
- **Arrastra pistas desde a [xanela da biblioteca](library.md)** a un
  cartucho para poñer a primeira nel (substitúe o seu ficheiro cando está
  parado). Un cartucho que está soando ou en CUE rexeita o que se solta.
- **Paralo todo** (extremo dereito da barra) para todos os cartuchos que están
  a soar, en todas as páxinas. A súa etiqueta mostra cantos soan, como en
  **Paralo todo (2)**; sen ningún en reprodución aparece atenuado e sen conta.
- Mentres un cartucho soa, o seu bordo e unha barra que medra desde a
  esquerda segundo soa toman a [cor](#setting-up-carts) do cartucho (vermello
  para un cartucho sen ela), unha liña fina marca o bordo xa reproducido
  sobre a forma de onda, e o seu tempo conta atrás. A liña móvese con
  suavidade, mesmo nun cartucho curto.
- Cando o ficheiro dun cartucho foi analizado, a súa forma de onda aparece
  tenuemente detrás do nome, para distinguir un son que entra de golpe doutro
  que entra en fundido. Cobre o que reproduce o cartucho, desde o seu cue-in
  ata o seu cue-out.
- Os cartuchos **superpóñense** de forma predeterminada: disparar un segundo
  non para o primeiro. Un cartucho configurado con **Parar os outros
  cartuchos ao disparalo** para primeiro todos os demais cartuchos en antena,
  en calquera páxina.
- Un cartucho configurado **En bucle** comeza de novo desde o seu cue-in
  cando chega ao seu final, sen pausa, ata que o pares.
- **Clic dereito** nun cartucho para ver máis opcións:

| Elemento | Acción |
|---|---|
| Preescoitar no CUE | Reprodúceo na saída CUE da cartucheira (atenuado cando a cartucheira non ten saída Cue). Na saída de antena pregunta primeiro mentres algo soa alí (ver [CUE](players.md#cue-pre-listen)) |
| Parar | Párao |
| Editar… | Ábreo en Configuración |

O menú dun cartucho baleiro só ten **Editar…**, para escoller o seu ficheiro.

- **Páxinas:** as lapelas ao carón de **CARTUCHEIRA** cambian de páxina. Un
  punto vermello indica que está a soar un cartucho desa páxina.
- Fai clic en **CARTUCHEIRA** para pregar a cartucheira ou despregala de
  novo. Pregada ao carón dos reprodutores é unha franxa estreita cunha frecha
  para despregala e, mentres soa un cartucho, un punto vermello.
- Baixo os reprodutores, tan alta como necesiten as súas filas, os botóns
  encollen cando a xanela é baixa (ata unha altura mínima) para que caiban
  todas as filas configuradas. Cando cambias o tamaño da cartucheira, ou
  ao carón dos reprodutores, os botóns medran para encherla, e os nomes e os
  tempos conservan o seu tamaño no centro de cada botón. En ambos os casos a
  cartucheira só se despraza cando nin os botóns máis pequenos caben.

| Aspecto do botón | Significado |
|---|---|
| Punto violeta | Jingle |
| Punto ámbar | Efecto |
| Punto gris | Cuña (publicidade) |
| Bordo, tinte e punto de cor | A cor do cartucho, escollida en Configuración: o punto toma a cor en lugar da do tipo |
| ↻ despois do tipo | En bucle |
| ✋ despois do tipo | Para os outros cartuchos ao dispararse |
| Ficheiro cunha cruz / sinal de aviso | O ficheiro falta / non se pode decodificar; pon o punteiro sobre o cartucho para ver a razón e a ruta. Un ficheiro que falta búscase de novo cada 30 s (`tuning.missing_recheck_ms`). |
| «Baleiro», atenuado | Sen ficheiro asignado |

Os cartuchos usan os mesmos marcadores que as pistas. Comezan no seu cue-in e
rematan no seu cue-out, que podes editar na forma de onda dun reprodutor
cando o ficheiro está cargado nel.

## Teclado {#keyboard}

De forma predeterminada **F1**…**F12** disparan os cartuchos 1–12 da páxina
que se ve, e **Ctrl+Space** para todos os cartuchos (o mesmo que **Paralo
todo**; tamén para un cartucho que estás preescoitando no CUE, aínda que
ningún cartucho estea soando). Consulta [Teclado](keyboard.md) para cambialos.

O atallo que pliega ou despliega a cartucheira non fai nada mentres a
cartucheira está oculta desde a barra superior (ver
[Disposición](layout.md#show-and-hide-panels)).

## Configurar os cartuchos {#setting-up-carts}

Vai a **Configuración → Cartucheira**:

- **Páxinas:** crear, renomear, eliminar (a última páxina non se pode
  eliminar) e definir o tamaño da grella (filas × columnas). Rexéitase unha
  grella máis pequena se fixese perder cartuchos que teñen ficheiro. Un
  tamaño escrito na caixa ao carón dun control deslizante aplícase ao premer
  Enter ou saír da caixa; Escape cancélao.
- **Importar… / Exportar…** gardan unha páxina nun ficheiro `.cartpage.json` e
  cárganna de novo, por exemplo para compartila entre estudos. As rutas
  relativas dos ficheiros resólvense a partir do cartafol do ficheiro. As
  cores viaxan coa páxina; un ficheiro dunha versión anterior impórtase sen
  elas.
- **Cartuchos:** fai clic nun cartucho da grella e despois define o seu nome,
  ficheiro (**Escoller…** ou **Quitar**), tipo, **Cor**, **En bucle** e
  **Parar os outros cartuchos ao disparalo**. **Cor** ofrece sen cor e oito
  cores (vermello, laranxa, ámbar, verde, turquesa, azul, violeta, rosa): dálle
  unha cor a todas as cuñas e outra aos indicativos da emisora para
  atopalos dun vistazo. Un cartucho baleiro conserva a súa cor, e quitar o
  ficheiro dun cartucho non llela retira. Un cartucho con ficheiro tamén
  ten **Excluír da normalización** e **Axuste**, os axustes de
  [normalización de sonoridade](loudness.md#per-track-exclude-and-trim) da súa
  pista, coa medición baixo a etiqueta.

As saídas Main e Cue propias da cartucheira están en **Configuración →
Saídas de audio** (a fila **Cartucheira**).
