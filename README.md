# 💽 DiskMan (Rust UDisks2 + Rofi)

Una utilidad ligera y ultrarrápida desarrollada en **Rust** para la gestión de volúmenes extraíbles en entornos Linux con Wayland y Hyprland (o cualquier gestor de ventanas). Diseñada para prescindir de pesados gestores de archivos y ofrecer control directo mediante menús flotantes, notificaciones nativas, apertura automática y monitorización en barras de estado.

## 🚀 Características

* **Rendimiento nativo:** Escrito en Rust, sin tiempos de espera ni consumo innecesario de recursos.
* **Integración con Rofi:** Despliega un menú interactivo en pantalla para montar y desmontar unidades con un solo atajo de teclado.
* **Filtrado Inteligente:** Ignora automáticamente las unidades padre no montables y lista únicamente las particiones o volúmenes extraíbles válidos.
* **Apertura Automática:** Lanza Nautilus de forma automática en una nueva ventana al montar con éxito un volumen.
* **Soporte para Waybar:** Modo nativo (`--waybar`) para mostrar en tiempo real cuántas unidades hay montadas y prevenir extracciones accidentales.
* **Sin Privilegios de Root:** Utiliza `udisksctl` y políticas de seguridad modernas (`polkit`) para operar de forma segura bajo el usuario actual.
* **Notificaciones de Escritorio:** Feedback visual instantáneo mediante `notify-send`.

---

## 🛠️ Requisitos del Sistema

* **Arch Linux** (o cualquier distro compatible con UDisks2).
* **Rust toolchain** (instalado vía `rustup`).
* **Rofi** (modo dmenu / Wayland fork compatible).
* **`udisksctl`** (suele venir preinstalado con el paquete `udisks2`).
* **`notify-send`** (paquete `libnotify`).
* **Nautilus** (explorador de archivos para la apertura automática al montar).

---

## ⚙️ Compilación e Instalación

1. Clona o sitúate en el directorio del proyecto:
   ```bash
   cd diskman
   ```

2. Compila el binario en modo optimizado de producción (*release*):
   ```bash
   cargo build --release
   ```

3. (Opcional) Copia el binario a tu ruta local de ejecutables:
   ```bash
   cp target/release/diskman ~/.local/bin/diskman
   chmod +x ~/.local/bin/diskman
   ```

---

## ⌨️ Integración con Hyprland

### Atajo de teclado para el menú de Rofi
Para invocar DiskMan con un atajo de teclado personalizado, añade la siguiente línea a tu archivo de configuración (`~/.config/hypr/hyprland.conf`):

```ini
# Ejemplo: Abrir DiskMan con Super + U 

hl.bind(mainMod .. "+ U", hl.dsp.exec_cmd("./.local/bin/diskman"))

```

### Integración con Waybar
Añade este bloque personalizado en tu configuración de Waybar (`~/.config/waybar/config.jsonc`) para monitorizar los discos activos:

```jsonc
"custom/diskman": {
    "exec": "~/.local/bin/diskman --waybar",
    "return-type": "json",
    "interval": 3,
    "format": "{}",
    "on-click": "~/.local/bin/diskman"
}
```

---

## 🧠 ¿Cómo funciona por debajo?

1. El programa ejecuta el comando del sistema `lsblk -J` para capturar la topología de bloques en formato JSON.
2. Utiliza las librerías `serde` y `serde_json` para deserializar el JSON de manera fuertemente tipada en estructuras de Rust.
3. Filtra los dispositivos extraíbles (`rm: true`) descartando las unidades principales e identificando si sus particiones se encuentran montadas o desmontadas.
4. Envía las opciones formateadas a **Rofi** mediante la entrada estándar (`stdin`) o genera el JSON correspondiente si se invoca con `--waybar`.
5. Captura la selección del usuario, ejecuta la acción con `udisksctl`, abre el explorador de archivos si procede y notifica el resultado del sistema.

---

## 📜 Licencia

Este proyecto es de código abierto y está disponible bajo los términos de la licencia MIT.
