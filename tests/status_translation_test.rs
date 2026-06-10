use speedtest_admin::filters::translate_status;

#[test]
fn test_translate_status_completed_en() {
    rust_i18n::set_locale("en");
    let result = translate_status("completed").unwrap();
    assert_eq!(result, "Completed");
}

#[test]
fn test_translate_status_failed_en() {
    rust_i18n::set_locale("en");
    let result = translate_status("failed").unwrap();
    assert_eq!(result, "Failed");
}

#[test]
fn test_translate_all_status_values_en() {
    rust_i18n::set_locale("en");
    
    assert_eq!(translate_status("benchmarking").unwrap(), "Benchmarking");
    assert_eq!(translate_status("checking").unwrap(), "Checking");
    assert_eq!(translate_status("completed").unwrap(), "Completed");
    assert_eq!(translate_status("failed").unwrap(), "Failed");
    assert_eq!(translate_status("running").unwrap(), "Running");
    assert_eq!(translate_status("started").unwrap(), "Started");
    assert_eq!(translate_status("skipped").unwrap(), "Skipped");
    assert_eq!(translate_status("waiting").unwrap(), "Waiting");
}

#[test]
fn test_translate_status_case_insensitive() {
    rust_i18n::set_locale("en");
    
    // Should handle uppercase
    let result = translate_status("COMPLETED").unwrap();
    assert_eq!(result, "Completed");
    
    let result2 = translate_status("Failed").unwrap();
    assert_eq!(result2, "Failed");
}

#[test]
fn test_translate_status_german() {
    rust_i18n::set_locale("de_DE");
    
    assert_eq!(translate_status("benchmarking").unwrap(), "Benchmarking");
    assert_eq!(translate_status("checking").unwrap(), "Prüfe");
    assert_eq!(translate_status("completed").unwrap(), "Abgeschlossen");
    assert_eq!(translate_status("failed").unwrap(), "Fehler");
    assert_eq!(translate_status("running").unwrap(), "Laufend");
    assert_eq!(translate_status("started").unwrap(), "Gestartet");
    assert_eq!(translate_status("skipped").unwrap(), "Übersprungen");
    assert_eq!(translate_status("waiting").unwrap(), "Warten");
}

#[test]
fn test_translate_status_spanish() {
    rust_i18n::set_locale("es_ES");
    
    assert_eq!(translate_status("benchmarking").unwrap(), "Evaluando");
    assert_eq!(translate_status("checking").unwrap(), "Comprobando");
    assert_eq!(translate_status("completed").unwrap(), "Completado");
    assert_eq!(translate_status("failed").unwrap(), "Fallido");
    assert_eq!(translate_status("running").unwrap(), "Ejecutando");
    assert_eq!(translate_status("started").unwrap(), "Iniciado");
    assert_eq!(translate_status("skipped").unwrap(), "Omitido");
    assert_eq!(translate_status("waiting").unwrap(), "Esperando");
}

#[test]
fn test_translate_status_french() {
    rust_i18n::set_locale("fr_FR");
    
    assert_eq!(translate_status("benchmarking").unwrap(), "Évaluation comparative");
    assert_eq!(translate_status("checking").unwrap(), "En cours de vérification");
    assert_eq!(translate_status("completed").unwrap(), "Terminé");
    assert_eq!(translate_status("failed").unwrap(), "Échec");
    assert_eq!(translate_status("running").unwrap(), "En cours d'exécution");
    assert_eq!(translate_status("started").unwrap(), "Démarré");
    assert_eq!(translate_status("skipped").unwrap(), "Ignoré");
    assert_eq!(translate_status("waiting").unwrap(), "En attente");
}

#[test]
fn test_translate_status_dutch() {
    rust_i18n::set_locale("nl_NL");
    
    assert_eq!(translate_status("benchmarking").unwrap(), "Benchmarking");
    assert_eq!(translate_status("checking").unwrap(), "Controleren");
    assert_eq!(translate_status("completed").unwrap(), "Voltooid");
    assert_eq!(translate_status("failed").unwrap(), "Mislukt");
    assert_eq!(translate_status("running").unwrap(), "Lopend");
    assert_eq!(translate_status("started").unwrap(), "Gestart");
    assert_eq!(translate_status("skipped").unwrap(), "Overgeslagen");
    assert_eq!(translate_status("waiting").unwrap(), "Wachten");
}

#[test]
fn test_translate_status_portuguese() {
    rust_i18n::set_locale("pt_BR");
    
    assert_eq!(translate_status("benchmarking").unwrap(), "Benchmarking");
    assert_eq!(translate_status("checking").unwrap(), "Verificando");
    assert_eq!(translate_status("completed").unwrap(), "Concluído");
    assert_eq!(translate_status("failed").unwrap(), "Falhou");
    assert_eq!(translate_status("running").unwrap(), "Executando");
    assert_eq!(translate_status("started").unwrap(), "Iniciado");
    assert_eq!(translate_status("skipped").unwrap(), "Ignorado");
    assert_eq!(translate_status("waiting").unwrap(), "Esperando");
}
