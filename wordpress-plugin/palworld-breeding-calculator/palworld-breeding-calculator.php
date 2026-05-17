<?php
/**
 * Plugin Name: Palworld Breeding Calculator
 * Description: Breeding calculator for WordPress (no Rust server required). Use shortcode [palworld_calculator].
 * Version: 1.0.0
 * Author: Palworld Breeding Calculator
 * License: GPL-2.0-or-later
 */

if (!defined('ABSPATH')) {
    exit;
}

define('PBC_VERSION', '1.0.0');
define('PBC_PLUGIN_DIR', plugin_dir_path(__FILE__));
define('PBC_PLUGIN_URL', plugin_dir_url(__FILE__));

function pbc_should_load_assets(): bool
{
    if (!is_singular()) {
        return false;
    }
    global $post;
    if (!$post) {
        return false;
    }
    return has_shortcode($post->post_content, 'palworld_calculator');
}

function pbc_enqueue_assets(): void
{
    if (!pbc_should_load_assets()) {
        return;
    }

    wp_enqueue_style(
        'pbc-calculator',
        PBC_PLUGIN_URL . 'assets/calculator.css',
        [],
        PBC_VERSION
    );
    wp_enqueue_script(
        'pbc-calculator',
        PBC_PLUGIN_URL . 'assets/calculator.js',
        [],
        PBC_VERSION,
        true
    );
    wp_localize_script('pbc-calculator', 'pbcConfig', [
        'palsUrl' => PBC_PLUGIN_URL . 'assets/pals.json',
        'combosUrl' => PBC_PLUGIN_URL . 'assets/special_combos.json',
    ]);
}
add_action('wp_enqueue_scripts', 'pbc_enqueue_assets');

function pbc_shortcode(): string
{
    ob_start();
    ?>
    <div class="pbc-wrap" id="pbc-app">
      <p class="pbc-lead">Select two parents or pick a target Pal for reverse combinations. Works fully on WordPress hosting.</p>
      <div class="pbc-grid">
        <label>Parent A
          <select id="pbc-parent-a"></select>
        </label>
        <label>Parent B
          <select id="pbc-parent-b"></select>
        </label>
        <label>Target child (reverse lookup)
          <select id="pbc-target"></select>
        </label>
      </div>
      <div class="pbc-actions">
        <button type="button" class="pbc-btn" id="pbc-calc-btn">Calculate child</button>
        <button type="button" class="pbc-btn pbc-btn-ghost" id="pbc-swap-btn">Swap parents</button>
        <button type="button" class="pbc-btn pbc-btn-ghost" id="pbc-combos-btn">Find combinations</button>
      </div>
      <div id="pbc-result" class="pbc-panel" aria-live="polite"></div>
      <div id="pbc-combos" class="pbc-panel"></div>
    </div>
    <?php
    return (string) ob_get_clean();
}
add_shortcode('palworld_calculator', 'pbc_shortcode');

function pbc_activate(): void
{
    $existing = get_page_by_path('palworld-breeding-calculator');
    if ($existing) {
        return;
    }

    wp_insert_post([
        'post_title' => 'Palworld Breeding Calculator',
        'post_name' => 'palworld-breeding-calculator',
        'post_content' => '[palworld_calculator]',
        'post_status' => 'publish',
        'post_type' => 'page',
    ]);
}
register_activation_hook(__FILE__, 'pbc_activate');
