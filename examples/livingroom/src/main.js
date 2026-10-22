// Living room real-time viewer — WebGPURenderer + node post-processing (GTAO + bloom).
// Tuned to match livingroom.blend: AgX view transform + 0.2 EV exposure,
// HDRI world strength 0.9, HDRI Z-rotation 335.6°, 42 W pendant area light.

import * as THREE from 'three/webgpu';
import { pass, mrt, normalView } from 'three/tsl';
import { ao } from 'three/addons/tsl/display/GTAONode.js';
import { bloom } from 'three/addons/tsl/display/BloomNode.js';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { DRACOLoader } from 'three/examples/jsm/loaders/DRACOLoader.js';
import { HDRLoader } from 'three/examples/jsm/loaders/HDRLoader.js';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { RectAreaLightTexturesLib } from 'three/examples/jsm/lights/RectAreaLightTexturesLib.js';

// Resolve asset URLs relative to the page itself, so this works both in the
// dev server and under a GitHub Pages subpath (https://…/<repo>/).
const assetURL = (p) => new URL(p, document.baseURI).href;
const GLB_URL = assetURL('export/livingroom.glb');
const DRACO_PATH = assetURL('export/draco/');
const HDRI_URL = assetURL('assets/hdri/kloofendal_38d_partly_cloudy_puresky_4k.hdr');

// Values read from livingroom.blend — see THREE_HANDOVER.md for the rest.
const BLEND_VIEW_EXPOSURE_EV = 0.2;      // scene.view_settings.exposure
const BLEND_WORLD_STRENGTH = 0.9;        // world Background node strength
const BLEND_WORLD_ROT_Z_DEG = 23;        // world Mapping node Z rotation
const DEFAULT_EXPOSURE = Math.pow(2, BLEND_VIEW_EXPOSURE_EV); // ~1.149
// The HDRI is a normalized (radiometric) map — sky ~0.15, sun ~0.8 — so all
// lights must live on that same scale. three/GLTF give us photometric units
// (candela, nits); dividing by 683 lm/W converts back to Blender's scale.
// Real-time can't occlude IBL with walls like Cycles, so indirect env is kept
// low and direct sun comes from a shadow-casting DirectionalLight.
const LUMEN_PER_WATT = 683;
const DEFAULT_ENV_INDIRECT = 0.35;
let sunMeasured = 1; // sun irradiance measured from the HDRI (see findSunDirection)

const statsEl = document.getElementById('stats');
const errorEl = document.getElementById('error');

let renderer, scene, camera, controls, renderPipeline, scenePass, aoPass, bloomPass;
let gltfScene, sunLight;
const sunBaseDir = new THREE.Vector3(1, 0.5, 0); // sun direction, un-rotated env space
const sunColorRGB = [1, 1, 1];
const exportCameras = {};
let activeCameraName = 'Cam_ThreeQuarter';

// fps bookkeeping
let frames = 0;
let fpsTime = 0;
window.__fps = 0;

init();

async function init() {

  if (!navigator.gpu) {
    showError('WebGPU is not available in this browser. Use Chrome/Edge — enable WebGPU in settings if needed.');
    return;
  }
  const adapter = await navigator.gpu.requestAdapter();
  if (!adapter) {
    showError('A WebGPU adapter could not be obtained (GPU blocked or unsupported). Enable WebGPU and try Chrome/Edge.');
    return;
  }

  try {

    // renderer — AgX matches Blender's view transform; exposure matches its +0.2 EV
    renderer = new THREE.WebGPURenderer({ antialias: true });
    await renderer.init();
    if (renderer.backend.isWebGPUBackend !== true) {
      showError('Three.js fell back to the WebGL2 backend — this viewer targets the WebGPU backend.');
      return;
    }
    renderer.toneMapping = THREE.AgXToneMapping;
    renderer.toneMappingExposure = DEFAULT_EXPOSURE;
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    renderer.localClippingEnabled = false;
    document.body.appendChild(renderer.domElement);

    scene = new THREE.Scene();

    // load GLB + HDRI in parallel
    setStats('Loading model (30 MB, Draco)…');
    const [gltf, hdr] = await Promise.all([loadGLB(), loadHDRI()]);

    gltfScene = gltf.scene;
    scene.add(gltfScene);

    // environment — same Poly Haven HDRI as the Blender world
    hdr.mapping = THREE.EquirectangularReflectionMapping;
    scene.environment = hdr;
    scene.background = hdr;
    scene.environmentIntensity = DEFAULT_ENV_INDIRECT;
    scene.backgroundIntensity = BLEND_WORLD_STRENGTH;
    scene.environmentRotation = new THREE.Euler(0, THREE.MathUtils.degToRad(BLEND_WORLD_ROT_Z_DEG), 0);
    scene.backgroundRotation = new THREE.Euler(0, THREE.MathUtils.degToRad(BLEND_WORLD_ROT_Z_DEG), 0);

    gltfScene.updateWorldMatrix(true, true);

    // GLTF exports punctual lights in photometric units (the floor lamp is
    // 1902 cd); convert to the HDRI's normalized radiometric scale
    gltfScene.traverse((obj) => {
      if (obj.isLight) obj.intensity /= LUMEN_PER_WATT;
    });

    // shadow flags — sun through the windows must cast real shadows
    gltfScene.traverse((obj) => {
      if (!obj.isMesh) return;
      const mat = obj.material;
      const skipCast = mat && (mat.transmission > 0 || mat.name === 'M_Curtain'
        || mat.name === 'M_BulbGlow' || mat.name === 'M_PaperShade');
      obj.castShadow = !skipCast;
      obj.receiveShadow = true;
    });

    // directional sun sampled from the HDRI's brightest texel, so it always
    // matches the visible sky and follows the env-rotation slider
    addSunLight(hdr);

    // collect the exported cameras (Cam_ThreeQuarter, Cam_ToWindow, Cam_Detail, Cam_Plant)
    gltfScene.traverse((obj) => {
      if (obj.isCamera && obj.name.startsWith('Cam_')) {
        exportCameras[obj.name] = obj;
      }
    });

    // glTF cannot carry the Cycles pendant area light — recreate it.
    // Blender: 42 W over a 0.16 m square, color (1.0, 0.63, 0.36) linear,
    // sitting at the bulb inside the paper shade, facing down.
    // three uses physical light units (confirmed by the exported floor lamp:
    // 35 W -> 1902 cd), so radiance = 42 / (0.16^2 * PI) W/m²/sr * 683 lm/W.
    addPendantAreaLight();

    // sheer curtain: alpha-blended doubleSided — keep it sorted last
    gltfScene.traverse((obj) => {
      const mat = obj.material;
      if (mat && mat.name === 'M_Curtain') {
        mat.depthWrite = false;
        obj.renderOrder = 10;
      }
    });

    // main render camera — starts as the exported hero camera
    camera = new THREE.PerspectiveCamera(45, 1, 0.025, 1000);

    // orbit controls
    controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.08;
    controls.maxDistance = 14;
    controls.minDistance = 0.05;

    setupPipeline();
    applyCameraPreset(activeCameraName, /* firstTime */ true);
    buildUI();

    // debug handles (used by console/CDP inspection)
    window.__renderer = renderer;
    window.__scene = scene;
    window.__hdr = hdr;
    window.__THREE = THREE;
    window.__camera = camera;
    window.__controls = controls;
    window.__cameras = exportCameras;
    window.__sun = sunLight;

    renderer.setAnimationLoop(animate);
    window.addEventListener('resize', onResize);
    onResize();

  } catch (err) {
    console.error(err);
    showError('Failed to start: ' + (err && err.message ? err.message : err));
  }

}

function setupPipeline() {

  renderPipeline = new THREE.RenderPipeline(renderer);

  // GTAO pre-pass: normals (view space) + depth via MRT.
  // samples: 0 — GTAO reads depth with textureGather, which needs a
  // single-sample depth attachment (the main scene pass keeps MSAA).
  const prePass = pass(scene, camera, { samples: 0 });
  prePass.setMRT(mrt({ output: normalView }));
  const prePassNormal = prePass.getTextureNode();
  const prePassDepth = prePass.getTextureNode('depth');

  scenePass = pass(scene, camera);
  const scenePassColor = scenePass.getTextureNode('output');

  aoPass = ao(prePassDepth, prePassNormal, camera);
  aoPass.radius.value = 0.35;    // meters — cozy-room scale
  aoPass.scale.value = 1.2;      // AO strength (0 = off)
  aoPass.distanceFallOff.value = 1.0;
  const aoTex = aoPass.getTextureNode();

  bloomPass = bloom(scenePassColor, 0.09, 0.4, 0.9); // strength, radius, threshold

  // tone-mapped scene color * AO, plus bloom of the bright lamps
  const aoColor = scenePassColor.mul(aoTex);
  renderPipeline.outputNode = aoColor.add(bloomPass);

  window.__pipeline = renderPipeline;
  window.__passes = { scenePass, aoPass, bloomPass, prePass };

}

function loadGLB() {

  return new Promise((resolve, reject) => {

    const draco = new DRACOLoader();
    draco.setDecoderPath(DRACO_PATH); // local r180 decoder from the export folder

    const loader = new GLTFLoader();
    loader.setDRACOLoader(draco);
    loader.load(
      GLB_URL,
      (gltf) => { draco.dispose(); resolve(gltf); },
      (evt) => {
        if (evt.lengthComputable) {
          setStats(`Loading model… ${Math.round((evt.loaded / evt.total) * 100)}%`);
        }
      },
      reject,
    );

  });

}

function loadHDRI() {

  setStats('Loading HDRI…');
  return new HDRLoader().loadAsync(HDRI_URL);

}

function addPendantAreaLight() {

  // WebGPU's LTC node implementation needs the BRDF approximation LUTs
  THREE.RectAreaLightNode.setLTC(RectAreaLightTexturesLib.init());

  const shade = gltfScene.getObjectByName('Pendant_Bulb') || gltfScene.getObjectByName('Pendant_Shade');
  const pos = shade
    ? shade.getWorldPosition(new THREE.Vector3())
    : new THREE.Vector3(-0.7, 1.66, -0.78); // Blender (-0.7, 0.78, 1.66) Z-up -> Y-up

  const light = new THREE.RectAreaLight(
    new THREE.Color().setRGB(1.0, 0.63, 0.36, THREE.LinearSRGBColorSpace),
    42 / (0.16 * 0.16 * Math.PI), // ~523 radiance — Blender scale, matches HDRI units
    0.16,
    0.16,
  );
  light.position.copy(pos);
  light.lookAt(pos.x, 0, pos.z); // face down, like the Blender area light
  scene.add(light);

}

function addSunLight(hdr) {

  sunMeasured = findSunDirection(hdr);

  const center = new THREE.Box3().setFromObject(gltfScene).getCenter(new THREE.Vector3());

  sunLight = new THREE.DirectionalLight(0xffffff, sunMeasured);
  sunLight.color.setRGB(sunColorRGB[0], sunColorRGB[1], sunColorRGB[2], THREE.LinearSRGBColorSpace);
  sunLight.castShadow = true;
  sunLight.shadow.mapSize.set(2048, 2048);
  sunLight.shadow.camera.near = 5;
  sunLight.shadow.camera.far = 60;
  const d = 9;
  sunLight.shadow.camera.left = -d;
  sunLight.shadow.camera.right = d;
  sunLight.shadow.camera.top = d;
  sunLight.shadow.camera.bottom = -d;
  sunLight.shadow.bias = -0.0004;
  sunLight.shadow.normalBias = 0.03;
  sunLight.target.position.copy(center);

  scene.add(sunLight, sunLight.target);
  positionSun();

}

function positionSun() {

  if (!sunLight) return;
  const dir = sunBaseDir.clone().applyEuler(scene.environmentRotation);
  const center = sunLight.target.position;
  sunLight.position.set(
    center.x + dir.x * 30,
    center.y + Math.max(dir.y, 0.02) * 30,
    center.z + dir.z * 30,
  );

}

// Scan the equirect HDRI for its brightest texel — that's the sun.
// Returns the sun's irradiance on a horizontal plane (in the HDRI's own
// normalized units); sets sunBaseDir and sunColorRGB as side effects.
function findSunDirection(hdr) {

  const w = hdr.image.width;
  const h = hdr.image.height;
  const data = hdr.image.data;
  const half = data instanceof Float32Array;

  const halfToFloat = (v) => {

    const s = (v & 0x8000) ? -1 : 1;
    const e = (v >> 10) & 0x1f;
    const f = v & 0x3ff;
    if (e === 0) return s * Math.pow(2, -14) * (f / 1024);
    if (e === 31) return f ? NaN : s * Infinity;
    return s * Math.pow(2, e - 15) * (1 + f / 1024);

  };

  let best = -1, bx = 0, by = 0;
  for (let y = 0; y < h; y += 2) {

    for (let x = 0; x < w; x += 2) {

      const i = (y * w + x) * 4;
      const r = half ? data[i] : halfToFloat(data[i]);
      const g = half ? data[i + 1] : halfToFloat(data[i + 1]);
      const b = half ? data[i + 2] : halfToFloat(data[i + 2]);
      const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
      if (lum > best) {

        best = lum;
        bx = x;
        by = y;

      }

    }

  }

  // three's equirect mapping: u = atan2(z, x)/2π + 0.5, v = asin(y)/π + 0.5
  const u = (bx + 0.5) / w;
  const v = (by + 0.5) / h;
  const theta = (u - 0.5) * Math.PI * 2;
  const y = Math.sin((v - 0.5) * Math.PI);
  const rXZ = Math.cos((v - 0.5) * Math.PI);
  sunBaseDir.set(Math.cos(theta) * rXZ, Math.max(y, 0.05), Math.sin(theta) * rXZ).normalize();

  const i = (by * w + bx) * 4;
  const cr = half ? data[i] : halfToFloat(data[i]);
  const cg = half ? data[i + 1] : halfToFloat(data[i + 1]);
  const cb = half ? data[i + 2] : halfToFloat(data[i + 2]);
  const m = Math.max(cr, cg, cb, 1e-6);
  sunColorRGB[0] = cr / m;
  sunColorRGB[1] = cg / m;
  sunColorRGB[2] = cb / m;

  // median sky luminance (coarse sample), then integrate the sun disc's
  // irradiance onto a horizontal plane in the same normalized units
  const lumAt = (j) => {

    const r = half ? data[j] : halfToFloat(data[j]);
    const g = half ? data[j + 1] : halfToFloat(data[j + 1]);
    const b = half ? data[j + 2] : halfToFloat(data[j + 2]);
    return 0.2126 * r + 0.7152 * g + 0.0722 * b;

  };

  const samples = [];
  for (let y = 0; y < h; y += 4) {
    for (let x = 0; x < w; x += 4) samples.push(lumAt((y * w + x) * 4));
  }
  samples.sort((a, b) => a - b);
  const sunThreshold = samples[samples.length >> 1] * 10;

  const dAz = (Math.PI * 2) / w;
  const dEl = Math.PI / h;
  let esun = 0;
  for (let y = 0; y < h; y += 2) {

    const el = ((y + 0.5) / h - 0.5) * Math.PI; // elevation
    const cosEl = Math.cos(el);
    if (cosEl <= 0) continue;
    for (let x = 0; x < w; x += 2) {

      const L = lumAt((y * w + x) * 4);
      if (L > sunThreshold) esun += L * cosEl * cosEl * dEl * dAz;

    }

  }

  return esun;

}

function applyCameraPreset(name, firstTime = false) {

  const src = exportCameras[name];
  if (!src) return;

  activeCameraName = name;

  src.updateWorldMatrix(true, false);
  src.matrixWorld.decompose(camera.position, camera.quaternion, new THREE.Vector3());

  camera.fov = src.fov;
  camera.near = 0.025;
  camera.far = 1000;
  camera.updateProjectionMatrix();

  // place the orbit pivot on the shot axis so the initial framing matches Blender
  const forward = new THREE.Vector3(0, 0, -1).applyQuaternion(camera.quaternion);
  const dist = firstTime ? 1.5 : controls ? controls.getDistance() : 1.5;
  controls.target.copy(camera.position).addScaledVector(forward, dist);
  controls.update();

}

function buildUI() {

  const el = (id) => document.getElementById(id);

  // camera list
  const select = el('cameraSelect');
  for (const name of Object.keys(exportCameras)) {
    const opt = document.createElement('option');
    opt.value = name;
    opt.textContent = name.replace('Cam_', '');
    if (name === activeCameraName) opt.selected = true;
    select.appendChild(opt);
  }
  select.addEventListener('change', () => applyCameraPreset(select.value));

  // exposure
  bindSlider('exposure', renderer.toneMappingExposure, 2, (v) => {
    renderer.toneMappingExposure = v;
  });

  // environment rotation (sun must enter the +Z window wall); sun follows it
  bindSlider('envRotation', BLEND_WORLD_ROT_Z_DEG, 1, (v) => {
    const rad = THREE.MathUtils.degToRad(v);
    scene.environmentRotation.set(0, rad, 0);
    scene.backgroundRotation.set(0, rad, 0);
    positionSun();
  }, '°');

  // indirect env light (Cycles occludes IBL by walls; raster cannot, so this
  // is dialed down and direct sun comes from the shadow-casting DirectionalLight)
  bindSlider('envIntensity', DEFAULT_ENV_INDIRECT, 2, (v) => {
    scene.environmentIntensity = v;
  });

  // direct sun strength (measured from the HDRI; units match the HDRI)
  bindSlider('sunIntensity', sunMeasured, 2, (v) => {
    sunLight.intensity = v;
  });

  // bloom
  bindSlider('bloomStrength', 0.09, 2, (v) => { bloomPass.strength.value = v; });
  bindSlider('bloomThreshold', 0.9, 2, (v) => { bloomPass.threshold.value = v; });
  bindSlider('bloomRadius', 0.4, 2, (v) => { bloomPass.radius.value = v; });

  // GTAO
  bindSlider('aoStrength', 1.2, 1, (v) => { aoPass.scale.value = v; });
  bindSlider('aoRadius', 0.35, 2, (v) => { aoPass.radius.value = v; }, ' m');

}

function bindSlider(id, initial, decimals, onChange, unit = '') {

  const el = (x) => document.getElementById(x);
  const slider = el(id);
  slider.value = initial;
  el(id + 'Val').textContent = Number(initial).toFixed(decimals) + unit;
  slider.addEventListener('input', () => {
    const v = parseFloat(slider.value);
    onChange(v);
    el(id + 'Val').textContent = v.toFixed(decimals) + unit;
  });

}

function animate() {

  const now = performance.now();
  frames++;
  if (now - fpsTime > 500) {
    window.__fps = Math.round((frames * 1000) / (now - fpsTime));
    frames = 0;
    fpsTime = now;
    updateHUD();
  }

  controls.update();
  renderPipeline.render();

}

function updateHUD() {

  const w = renderer.domElement.width;
  const h = renderer.domElement.height;
  statsEl.textContent = `${window.__fps} fps · ${w}×${h}\nAgX · exposure ${renderer.toneMappingExposure.toFixed(2)} · env ${scene.environmentIntensity.toFixed(2)}`;

}

function onResize() {

  const w = window.innerWidth;
  const h = window.innerHeight;
  renderer.setSize(w, h);
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  camera.aspect = w / h;
  camera.updateProjectionMatrix();

}

function setStats(text) {

  statsEl.textContent = text;

}

function showError(text) {

  window.__loadError = text;
  errorEl.style.display = 'grid';
  errorEl.textContent = text;

}
