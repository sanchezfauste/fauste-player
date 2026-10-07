# Cartucheira

A cartucheira é a franxa de botóns baixo os reprodutores. Cada botón, un
**cartucho**, reproduce un son ao instante: jingles, efectos, cuñas. Os
cartuchos soan nas súas propias saídas, con independencia dos reprodutores.

![A cartucheira cun cartucho en reprodución](../../images/guide/cartwall.png)

## Uso {#using-it}

- **Fai clic nun cartucho** para disparalo. **Fai clic nel de novo** para
  paralo.
- **Paralo todo** (extremo dereito da barra) para todos os cartuchos que están
  a soar, en todas as páxinas. A súa etiqueta mostra cantos soan, como en
  **Paralo todo (2)**; sen ningún en reprodución aparece atenuado e sen conta.
- Mentres un cartucho soa, o seu bordo vólvese vermello, unha barra vermella
  vai encollendo e o seu tempo conta atrás.
- Os cartuchos **superpóñense** de forma predeterminada: disparar un segundo
  non para o primeiro. Un cartucho configurado con **Parar os outros
  cartuchos ao disparalo** para primeiro todos os demais cartuchos en antena,
  en calquera páxina.
- Un cartucho configurado **En bucle** comeza de novo desde o seu cue-in
  cando chega ao seu final, sen pausa, ata que o pares.
- **Clic dereito** nun cartucho para ver máis opcións:

| Elemento | Acción |
|---|---|
| Preescoitar no CUE | Reprodúceo na saída CUE da cartucheira (atenuado cando a cartucheira non ten saída Cue á parte da súa Main) |
| Parar | Párao |
| Editar… | Ábreo en Configuración |

O menú dun cartucho baleiro só ten **Editar…**, para escoller o seu ficheiro.

- **Páxinas:** as lapelas ao carón de **CARTUCHEIRA** cambian de páxina. Un
  punto vermello indica que está a soar un cartucho desa páxina.
- Fai clic en **CARTUCHEIRA** para pregar a franxa ou despregala de novo.
- Cando a xanela é baixa, os botóns encollen (ata unha altura mínima) para
  que caiban todas as filas configuradas; a cartucheira só se despraza cando
  nin os botóns máis pequenos caben.

| Aspecto do botón | Significado |
|---|---|
| Punto violeta | Jingle |
| Punto ámbar | Efecto |
| Punto gris | Cuña (publicidade) |
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

## Configurar os cartuchos {#setting-up-carts}

Vai a **Configuración → Cartucheira**:

- **Páxinas:** crear, renomear, eliminar (a última páxina non se pode
  eliminar) e definir o tamaño da grella (filas × columnas). Rexéitase unha
  grella máis pequena se fixese perder cartuchos que teñen ficheiro.
- **Importar… / Exportar…** gardan unha páxina nun ficheiro `.cartpage.json` e
  cárganna de novo, por exemplo para compartila entre estudos. As rutas
  relativas dos ficheiros resólvense a partir do cartafol do ficheiro.
- **Cartuchos:** fai clic nun cartucho da grella e despois define o seu nome,
  ficheiro (**Escoller…** ou **Quitar**), tipo, **En bucle** e **Parar os
  outros cartuchos ao disparalo**.

As saídas Main e Cue propias da cartucheira están en **Configuración →
Saídas de audio** (a fila **Cartucheira**).
