use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Deserialize, Debug)]
struct LsblkOutput {
    blockdevices: Vec<Dispositivo>,
}

#[derive(Deserialize, Debug)]
struct Dispositivo {
    name: String,
    size: String,
    rm: bool,
    #[serde(default)]
    mountpoints: Option<Vec<Option<String>>>,
    #[serde(default)]
    children: Option<Vec<Dispositivo>>,
}

#[derive(Debug, Clone)]
struct InfoDispositivo {
    name: String,
    size: String,
    montado: bool,
}

#[derive(Serialize)]
struct WaybarOutput {
    text: String,
    tooltip: String,
    alt: String,
    class: Vec<String>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let modo_waybar = args.iter().any(|arg| arg == "--waybar");
    let modo_flash = args.iter().any(|arg| arg == "--flash");

    let dispositivos = match listar_extraibles() {
        Ok(lista) => lista,
        Err(e) => {
            if modo_waybar {
                let error_json = WaybarOutput {
                    text: String::from("  Err"),
                    tooltip: format!("Error al leer dispositivos: {}", e),
                    alt: String::from("error"),
                    class: vec![String::from("error")],
                };
                println!("{}", serde_json::to_string(&error_json).unwrap());
            } else {
                enviar_notificacion("DiskMan", &format!("Error al listar discos: {}", e), "critical");
            }
            return;
        }
    };

    if modo_waybar {
        generar_salida_waybar(&dispositivos);
        return;
    }

    if modo_flash {
        if dispositivos.is_empty() {
            enviar_notificacion("DiskMan", "No hay dispositivos USB conectados para flashear.", "normal");
            return;
        }
        ejecutar_flasheo(&dispositivos);
        return;
    }

    if dispositivos.is_empty() {
        enviar_notificacion("DiskMan", "No hay dispositivos extraíbles conectados.", "normal");
        return;
    }

    let mut opciones_texto = String::new();
    for d in &dispositivos {
        let accion = if d.montado { "Desmontar 󰚦" } else { "Montar 󰚥" };
        let estado = if d.montado { "Montado" } else { "Desmontado" };
        let linea = format!("{} ➔ /dev/{} ({}) [{}]\n", accion, d.name, d.size, estado);
        opciones_texto.push_str(&linea);
    }

    let seleccion = match lanzar_rofi_custom(&opciones_texto, "  Almacenamiento") {
        Ok(res) => res,
        Err(_) => return,
    };

    if seleccion.trim().is_empty() {
        return;
    }

    if let Some(dispositivo) = extraer_nombre_dispositivo(&seleccion) {
        let es_desmontar = seleccion.starts_with("Desmontar");
        let accion_str = if es_desmontar { "unmount" } else { "mount" };

        match gestionar_volumen(&dispositivo, accion_str) {
            Ok(_) => {
                let msg = format!(
                    "Dispositivo /dev/{} {} con éxito.",
                    dispositivo,
                    if es_desmontar { "desmontado" } else { "montado" }
                );
                enviar_notificacion("DiskMan", &msg, "normal");
            }
            Err(e) => {
                let msg = format!("Fallo al gestionar /dev/{}: {}", dispositivo, e);
                enviar_notificacion("DiskMan", &msg, "critical");
            }
        }
    }
}


fn listar_extraibles() -> Result<Vec<InfoDispositivo>, Box<dyn std::error::Error>> {
    let output = Command::new("lsblk").arg("-J").output()?;
    let datos: LsblkOutput = serde_json::from_slice(&output.stdout)?;

    let mut lista = Vec::new();

    for disco in datos.blockdevices {
        if let Some(hijos) = disco.children {
            for hijo in hijos {
                if hijo.rm {
                    lista.push(mapear_dispositivo(&hijo));
                }
            }
        } else if disco.rm {
            lista.push(mapear_dispositivo(&disco));
        }
    }

    Ok(lista)
}

fn mapear_dispositivo(d: &Dispositivo) -> InfoDispositivo {
    let montado = match &d.mountpoints {
        Some(pts) => pts.iter().any(|p| p.is_some()),
        None => false,
    };

    InfoDispositivo {
        name: d.name.clone(),
        size: d.size.clone(),
        montado,
    }
}

fn extraer_nombre_dispositivo(linea_rofi: &str) -> Option<String> {
    if let Some(idx) = linea_rofi.find("/dev/") {
        let resto = &linea_rofi[idx + 5..];
        let nombre: String = resto.chars().take_while(|c| c.is_alphanumeric()).collect();
        if !nombre.is_empty() {
            return Some(nombre);
        }
    }
    None
}

fn gestionar_volumen(nombre_particion: &str, accion: &str) -> Result<(), String> {
    let dispositivo_path = format!("/dev/{}", nombre_particion);

    let status = Command::new("udisksctl")
        .arg(accion)
        .arg("-b")
        .arg(&dispositivo_path)
        .status()
        .map_err(|e| format!("Error al invocar udisksctl: {}", e))?;

    if !status.success() {
        return Err(String::from("udisksctl devolvió un error (¿falta de permisos o contraseña de polkit?)"));
    }

    if accion == "mount" {
        if let Ok(user) = std::env::var("USER") {
            let ruta_montaje = format!("/run/media/{}", user);
            let _ = Command::new("nautilus")
                .arg("--new-window")
                .arg(ruta_montaje)
                .spawn();
        }
    }

    Ok(())
}

fn generar_salida_waybar(dispositivos: &[InfoDispositivo]) {
    let montados: Vec<&InfoDispositivo> = dispositivos.iter().filter(|d| d.montado).collect();
    let count = montados.len();

    if count == 0 {
        let salida = WaybarOutput {
            text: String::from("  0"),
            tooltip: String::from("No hay dispositivos extraíbles montados."),
            alt: String::from("unmounted"),
            class: vec![String::from("unmounted")],
        };
        println!("{}", serde_json::to_string(&salida).unwrap());
    } else {
        let mut tooltip_lines = String::from("Dispositivos montados:\n");
        for d in &montados {
            tooltip_lines.push_str(&format!("• /dev/{} ({})\n", d.name, d.size));
        }

        let salida = WaybarOutput {
            text: format!("  {}", count),
            tooltip: tooltip_lines.trim().to_string(),
            alt: String::from("mounted"),
            class: vec![String::from("mounted"), String::from("warning")],
        };
        println!("{}", serde_json::to_string(&salida).unwrap());
    }
}


fn lanzar_rofi_custom(opciones: &str, prompt: &str) -> Result<String, std::io::Error> {
    let mut child = Command::new("rofi")
        .args(["-dmenu", "-i", "-p", prompt, "-no-custom"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(opciones.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn enviar_notificacion(titulo: &str, mensaje: &str, urgencia: &str) {
    let _ = Command::new("notify-send")
        .arg("-u")
        .arg(urgencia)
        .arg(titulo)
        .arg(mensaje)
        .status();
}

fn ejecutar_flasheo(dispositivos: &[InfoDispositivo]) {
    let output_find = Command::new("find")
        .args(["/home", "-maxdepth", "4", "-name", "*.iso"])
        .output()
        .expect("Fallo al buscar ISOs");
    
    let isos = String::from_utf8_lossy(&output_find.stdout).to_string();
    if isos.trim().is_empty() {
        enviar_notificacion("DiskMan Flash", "No se encontraron archivos .iso en tu directorio personal.", "normal");
        return;
    }

    let iso_seleccionada = match lanzar_rofi_custom(&isos, "💿 Seleccionar ISO") {
        Ok(res) if !res.trim().is_empty() => res.trim().to_string(),
        _ => return,
    };

    let mut opciones_usb = String::new();
    for d in dispositivos {
        let disco_base: String = d.name.chars().take_while(|c| c.is_alphabetic()).collect();
        opciones_usb.push_str(&format!("/dev/{} ➔ ({})\n", disco_base, d.size));
    }

    let usb_seleccionado = match lanzar_rofi_custom(&opciones_usb, "⚠️ DESTINO (Se borrarán los datos)") {
        Ok(res) if !res.trim().is_empty() => res,
        _ => return,
    };

    let destino: String = usb_seleccionado.split_whitespace().next().unwrap_or("").to_string();
    if destino.is_empty() { return; }

    let confirmacion = lanzar_rofi_custom("NO\nSÍ, BORRAR TODO", &format!("¿Flashear {} en {}?", iso_seleccionada, destino));
    
    if let Ok(resp) = confirmacion {
        if resp.trim() == "SÍ, BORRAR TODO" {
            let _ = Command::new("kitty")
                .args([
                    "--class", "diskman-flash",
                    "-e", "sudo", "dd",
                    &format!("if={}", iso_seleccionada),
                    &format!("of={}", destino),
                    "bs=4M",
                    "status=progress",
                    "oflag=sync"
                ])
                .spawn();
        } else {
            enviar_notificacion("DiskMan Flash", "Flasheo cancelado por el usuario.", "normal");
        }
    }
}
