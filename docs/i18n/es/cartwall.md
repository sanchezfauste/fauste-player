# Cartuchera

La cartuchera es la franja de botones que hay debajo de los players. Cada
botón, un **cartucho**, reproduce un sonido al instante: jingles, efectos,
cuñas. Los cartuchos suenan por sus propias salidas, independientemente de
los players.

![La cartuchera con un cartucho sonando](../../images/guide/cartwall.png)

## Uso {#using-it}

- **Haz clic en un cartucho** para dispararlo. **Vuelve a hacer clic** para
  pararlo.
- **Parar todo** (en el extremo derecho de la barra) detiene todos los
  cartuchos que suenan, en todas las páginas. Su etiqueta muestra cuántos
  suenan, como en **Parar todo (2)**; si no suena ninguno, aparece atenuado
  y sin número.
- Mientras un cartucho suena, su borde se pone rojo, una barra roja se va
  acortando y su tiempo cuenta hacia atrás.
- Por defecto los cartuchos **se solapan**: disparar un segundo cartucho no
  detiene el primero. Un cartucho con **Parar los demás al dispararlo**
  detiene antes todos los demás cartuchos que suenan, en cualquier página.
- Un cartucho con **Repetir en bucle** vuelve a empezar desde su cue-in al
  llegar al final, sin hueco, hasta que lo paras.
- **Haz clic derecho** en un cartucho para ver más opciones:

| Elemento | Acción |
|---|---|
| Preescuchar en CUE | Lo reproduce en la salida CUE de la cartuchera (atenuado si la cartuchera no tiene una salida Cue distinta de su salida Main) |
| Parar | Lo detiene |
| Editar… | Lo abre en Configuración |

El menú de un cartucho vacío solo tiene **Editar…**, para elegir su archivo.

- **Páginas:** las pestañas junto a **CARTUCHERA** cambian de página. Un
  punto rojo indica que suena un cartucho de esa página.
- Haz clic en **CARTUCHERA** para plegar la franja o volver a desplegarla.
- Cuando la ventana es baja, los botones se encogen (hasta una altura
  mínima) para que quepan todas las filas configuradas; la cartuchera solo
  se desplaza cuando ni siquiera caben los botones más pequeños.

| Aspecto del botón | Significado |
|---|---|
| Punto violeta | Jingle |
| Punto ámbar | Efecto |
| Punto gris | Cuña (publicidad) |
| ↻ después del tipo | Se repite en bucle |
| ✋ después del tipo | Detiene los demás cartuchos al dispararlo |
| Archivo con una cruz / señal de aviso | El archivo falta / no se puede decodificar; pasa el puntero por el cartucho para ver el motivo y la ruta. Un archivo que falta se vuelve a buscar cada 30 s (`tuning.missing_recheck_ms`). |
| «Vacío», atenuado | Sin archivo asignado |

Los cartuchos usan los mismos marcadores que las pistas. Empiezan en su
cue-in y acaban en su cue-out, que puedes editar en la forma de onda de un
player cuando el archivo está cargado allí.

## Teclado {#keyboard}

Por defecto **F1**…**F12** disparan los cartuchos 1–12 de la página que se
ve, y **Ctrl+Espacio** detiene todos los cartuchos (igual que **Parar
todo**; también detiene un cartucho que estés preescuchando en CUE, aunque
no suene ningún cartucho). Consulta [Teclado](keyboard.md) para cambiarlos.

## Configurar los cartuchos {#setting-up-carts}

Ve a **Configuración → Cartuchera**:

- **Páginas:** crea, renombra y elimina páginas (la última no se puede
  eliminar), y fija el tamaño de la rejilla (filas × columnas). Se rechaza
  una rejilla más pequeña si quitaría cartuchos que tienen archivo.
- **Importar… / Exportar…** guardan una página en un archivo
  `.cartpage.json` y la vuelven a cargar, por ejemplo para compartirla entre
  estudios. Las rutas de archivo relativas se resuelven respecto a la
  carpeta del archivo.
- **Cartuchos:** haz clic en un cartucho de la rejilla y luego fija su
  nombre, su archivo (**Elegir…** o **Quitar**), su tipo, **Repetir en
  bucle** y **Parar los demás al dispararlo**.

Las salidas Main y Cue propias de la cartuchera están en **Configuración →
Salidas de audio** (la fila **Cartuchera**).
