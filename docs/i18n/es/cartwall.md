# Cartuchera

La cartuchera es el panel de botones que hay debajo de los players, o a su
lado (véase [Distribución](layout.md#move-and-resize-the-cartwall)). Cada
botón, un **cartucho**, reproduce un sonido al instante: jingles, efectos,
cuñas. Los cartuchos suenan por sus propias salidas, independientemente de
los players.

![La cartuchera con un cartucho sonando](../../images/guide/cartwall.png)

## Uso {#using-it}

- **Haz clic en un cartucho** para dispararlo. **Vuelve a hacer clic** para
  pararlo.
- **Arrastra pistas desde la [ventana de la biblioteca](library.md)** a un
  cartucho para poner la primera en él (sustituyendo su archivo cuando está
  parado). Un cartucho que suena o está en CUE rechaza el soltado.
- **Parar todo** (en el extremo derecho de la barra) detiene todos los
  cartuchos que suenan, en todas las páginas. Su etiqueta muestra cuántos
  suenan, como en **Parar todo (2)**; si no suena ninguno, aparece atenuado
  y sin número.
- Mientras un cartucho suena, su borde y una barra que crece desde la
  izquierda mientras suena toman el [color](#setting-up-carts) del cartucho
  (rojo si no tiene ninguno), una línea fina marca el borde reproducido
  sobre la forma de onda y su tiempo cuenta hacia atrás. La línea se mueve
  con suavidad, incluso en un cartucho corto.
- Cuando el archivo de un cartucho se ha analizado, su forma de onda se ve
  tenue detrás de su nombre, para distinguir un sonido que golpea de
  inmediato de uno que entra poco a poco. Cubre lo que reproduce el
  cartucho, desde su cue-in hasta su cue-out.
- Por defecto los cartuchos **se solapan**: disparar un segundo cartucho no
  detiene el primero. Un cartucho con **Parar los demás al dispararlo**
  detiene antes todos los demás cartuchos que suenan, en cualquier página.
- Un cartucho con **Repetir en bucle** vuelve a empezar desde su cue-in al
  llegar al final, sin hueco, hasta que lo paras.
- **Haz clic derecho** en un cartucho para ver más opciones:

| Elemento | Acción |
|---|---|
| Preescuchar en CUE | Lo reproduce en la salida CUE de la cartuchera (atenuado si la cartuchera no tiene salida Cue). En la salida en antena pregunta primero si algo suena allí (véase [CUE](players.md#cue-pre-listen)) |
| Parar | Lo detiene |
| Editar… | Lo abre en Configuración |

El menú de un cartucho vacío solo tiene **Editar…**, para elegir su archivo.

- **Páginas:** las pestañas junto a **CARTUCHERA** cambian de página. Un
  punto rojo indica que suena un cartucho de esa página.
- Haz clic en **CARTUCHERA** para plegar la cartuchera o volver a
  desplegarla. Plegada junto a los players es una franja estrecha con una
  flecha para desplegarla y, mientras suena un cartucho, un punto rojo.
- Debajo de los players, con la altura que necesitan sus filas, los botones
  se encogen cuando la ventana es baja (hasta una altura mínima) para que
  quepan todas las filas configuradas. Cuando cambias el tamaño de la
  cartuchera, o junto a los players, los botones crecen para llenarla, y
  los nombres y los tiempos conservan su tamaño en el centro de cada
  botón. En ambos casos la cartuchera solo se desplaza cuando ni siquiera
  caben los botones más pequeños.

| Aspecto del botón | Significado |
|---|---|
| Punto violeta | Jingle |
| Punto ámbar | Efecto |
| Punto gris | Cuña (publicidad) |
| Borde, tinte y punto de color | El color del cartucho, elegido en Configuración: el punto toma ese color en lugar del del tipo |
| ↻ después del tipo | Se repite en bucle |
| ✋ después del tipo | Detiene los demás cartuchos al dispararlo |
| Archivo con una cruz / señal de aviso | El archivo falta / no se puede decodificar; pasa el puntero por el cartucho para ver el motivo y la ruta. Un archivo que falta se vuelve a buscar cada 30 s (`tuning.missing_recheck_ms`). |
| «Vacío», atenuado | Sin archivo asignado |

Los cartuchos usan los mismos marcadores que las pistas. Empiezan en su
cue-in y acaban en su cue-out, que puedes editar en la forma de onda de un
player cuando el archivo está cargado allí.

## Teclado {#keyboard}

Por defecto **F1**…**F12** disparan los cartuchos 1–12 de la página que se
ve, y **Ctrl+Space** detiene todos los cartuchos (igual que **Parar
todo**; también detiene un cartucho que estés preescuchando en CUE, aunque
no suene ningún cartucho). Consulta [Teclado](keyboard.md) para cambiarlos.

El atajo que pliega o despliega la cartuchera no hace nada mientras la
cartuchera está oculta desde la barra superior (véase
[Distribución](layout.md#show-and-hide-panels)).

## Configurar los cartuchos {#setting-up-carts}

Ve a **Configuración → Cartuchera**:

- **Páginas:** crea, renombra y elimina páginas (la última no se puede
  eliminar), y fija el tamaño de la rejilla (filas × columnas). Se rechaza
  una rejilla más pequeña si quitaría cartuchos que tienen archivo. Un
  tamaño escrito en la casilla junto a un deslizador se aplica al pulsar
  `Enter` o al salir de la casilla; `Esc` lo cancela.
- **Importar… / Exportar…** guardan una página en un archivo
  `.cartpage.json` y la vuelven a cargar, por ejemplo para compartirla entre
  estudios. Las rutas de archivo relativas se resuelven respecto a la
  carpeta del archivo. Los colores viajan con la página; un archivo de una
  versión anterior se importa sin ellos.
- **Cartuchos:** haz clic en un cartucho de la rejilla y luego fija su
  nombre, su archivo (**Elegir…** o **Quitar**), su tipo, su **Color**,
  **Repetir en bucle** y **Parar los demás al dispararlo**. **Color** ofrece
  sin color y ocho colores (rojo, naranja, ámbar, verde, turquesa, azul,
  violeta, rosa): da a todas las cuñas un color y a las identificaciones de
  emisora otro para encontrarlas de un vistazo. Un cartucho vacío conserva
  su color, y quitar el archivo de un cartucho no lo elimina. Un cartucho
  con archivo tiene también **Excluir de la normalización** y **Ajuste
  manual**, los ajustes de [normalización de la
  sonoridad](loudness.md#per-track-exclude-and-trim) de su pista, con la
  medición bajo la etiqueta.

Las salidas Main y Cue propias de la cartuchera están en **Configuración →
Salidas de audio** (la fila **Cartuchera**).
