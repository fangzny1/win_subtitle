<script setup lang="ts">
import { nextTick, ref } from 'vue';
import { listen } from "@tauri-apps/api/event"
import { invoke } from "@tauri-apps/api/core"

interface Row {
  id: number;
  text: string;
  trans: string;
}
interface Cfg {   // 首字母大写，这是 TS 的规矩；另注意 enable_cuda 是 boolean 不是 number
  vad_slience_ms: number; vad_max_speech_ms: number; provider: string;
  enable_cuda: boolean; ai_api_base: string; ai_api_key: string;
  ai_model: string; target_lang: string;
}
const items = ref<Row[]>([]);
const listEl = ref<HTMLElement>();
const showSettings =ref(false)

const scrollBottom = () => nextTick(() => {
  if (listEl.value) listEl.value.scrollTop = listEl.value.scrollHeight;
});

listen<{ id: number; text: string }>("subtitle", (e) => {
  items.value.push({ id: e.payload.id, text: e.payload.text, trans: "翻译中…" });
  scrollBottom();
});
listen<{ id: number; trans: string }>("translation", (e) => {
  const item = items.value.find(i => i.id === e.payload.id);
  if (item) item.trans = e.payload.trans;
  scrollBottom();
});
const from =ref<Cfg>({ vad_slience_ms: 800, vad_max_speech_ms: 5000, provider: "cpu",
  enable_cuda: false, ai_api_base: "", ai_api_key: "", ai_model: "", target_lang: "zh" })
async function openSettings() {
  showSettings.value=true
  from.value =await invoke<Cfg>("get_config")
}


const running =ref(false);
const busy =ref(false)
const toggle =async ()=>{
  if (busy.value)return;
  busy.value =true
  try {
    if (!running.value)
  {
    await invoke("start_loopback")
    running.value=true
  }
  else{
    await invoke("stop_loopback");
    running.value=false
  }
  } catch (error) {
     console.log(error);
     
  }
  finally{
    busy.value =false
  }
}
async function save() {
   try{await invoke("save_config",{cfg:from.value});
   showSettings.value=false}
   catch(e){
     console.log("保存失败:", e)
     
   }

} 
const show =ref(false)
async function openWindow() {
  try {
    
    await invoke("set_decorations",{show:!show.value})
    show.value= !show.value
    
  } catch (error) {
    console.log(error);
    
  }
}
function cleanitem(){
  items.value=[];
}

</script>

<template>
  <div data-tauri-drag-region class="container">
    <div ref="listEl" class="list" :class="{blur:showSettings}">
      <div v-if="items.length === 0" class="hint">等待字幕… 按 Start 开始</div>
      <div v-for="it in items" :key="it.id" class="row">
        <div class="src">{{ it.text }}</div>
        <div class="dst">{{ it.trans }}</div>
      </div>
    </div >
   <div v-if="showSettings" class="settings">
         <input v-model="from.ai_api_base">
      <input v-model="from.ai_model" />
      <input v-model="from.ai_api_key">
      <input v-model="from.target_lang">
      <input v-model.number="from.vad_max_speech_ms">
      <input v-model.number="from.vad_slience_ms">
         <button @click="save()">save</button>
         <button @click="showSettings =false">exit</button>
         <button @click="openWindow()">开启标题栏</button>
         <button @click="cleanitem()">清理字幕</button>
        </div>
    <div class="bar">
    <button @click="openSettings" class="settings_btn">⚙</button>
    <button @click="toggle" class="btn" :disabled="busy" :class="{stop: running}">{{ running? "stop":"start" }}</button>
    </div>
  </div>
</template>

<style>
html, body { margin: 0; }
.container {
  background: rgba(0, 0, 0, 0.6);
  color: #fff;
  height: 100vh;
  display: flex;
  flex-direction: column;
}
.list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 10px;
}
.list.blur{
  filter: blur(3px);
}
.hint { color: #888; }
.row { margin-bottom: 8px; }
.src { color: #fff; font-size: 16px; line-height: 1.4; }
.dst { color: #0f0; font-size: 14px; line-height: 1.4; }
.btn { margin: 6px; padding: 6px 0; cursor: pointer; border-radius: 999px;}
.bar{display: flex;gap: 8px;padding: 6px;align-items: center;}
.btn{flex:1}
.btn.stop
{
  background-color: #ff050513;
  color: rgba(255, 255, 255, 0.473);
  backdrop-filter: blur(10px);
  
}
.settings{
   position: fixed;
   inset: 0;
   background: rgba(0,0,0,.7);
}
.settings_btn{
  margin-left: 20px;
  width: 40px;
  border-radius: 999px;
  background-color: rgba(255, 255, 255, 0.644);
  backdrop-filter: blur(12px);
}
.list::-webkit-scrollbar{
  width: 6px;
}
.list::-webkit-scrollbar-thumb{
  background: rgba(255,255,255,.25); border-radius: 999px;
}
.list::-webkit-scrollbar-track{
  background: transparent;
}
</style>
