<script setup lang="ts">
import { nextTick, ref } from 'vue';
import { listen } from "@tauri-apps/api/event"
import { invoke } from "@tauri-apps/api/core"
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs';
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
async function save_c() {
   try{await invoke("save_config",{cfg:from.value});
   showSettings.value=false}
   catch(e){
     console.log("保存失败:", e)
     
   }

} 
async function exportText() {
  try {
      const path =await save({filters: [{
    name: 'export',
    extensions: ['txt']
  }]})
  if(!path)return;
const content = items.value.map((it,t )=> `${t+1}. ${it.text}\n${it.trans}`).join("\n\n");
  await writeTextFile(path,content)
  } catch (error) {
    console.log(error);
    
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
    <div class="settings-main">
     <label class="label-settings">API 端点    <input v-model="from.ai_api_base" class="input-settings"></label>
     <label class="label-settings">模型 ID<input v-model="from.ai_model" class="input-settings"/></label> 
     <label class="label-settings" >API Key <input v-model="from.ai_api_key" type="password" class="input-settings"></label>
    <label class="label-settings">目标语言<input v-model="from.target_lang"class="input-settings"></label> 
     <label class="label-settings">最大切分时长<input v-model.number="from.vad_max_speech_ms"class="input-settings">ms</label>
    <label class="label-settings">静音时超时切分时长<input v-model.number="from.vad_slience_ms"class="input-settings">ms</label> 
    </div>
          <div class="list-button">
     <button class="button-settings" @click="save_c()">save</button>
         <button class="button-settings" @click="showSettings =false">exit</button>
         <button class="button-settings" @click="openWindow()">{{ show? "关闭标题栏":"开启标题栏" }}</button>
         <button class="button-settings" @click="cleanitem()">清理字幕</button>
         <button class="button-settings" @click="exportText()">导出字幕</button>
         </div>
        </div>
    <div class="bar" :class="{blur:showSettings}">
    <button @click="openSettings" class="settings_btn">⚙</button>
    <button @click="toggle" class="btn" :disabled="busy" :class="{stop: running}">{{ running? "stop":"start" }}</button>
    </div>
  </div>
</template>

<style>
html, body { margin: 0; }
.list-button{
  display: flex;
  padding: 0px 15px;
  gap: 15px;
  justify-content: center;
overflow: visible;

  
}
.button-settings{
  color: white;
  padding: 5px;
  width: 90px;
  background-color: #5eff006c;
  outline: none;
  box-shadow:  0 0 8px rgba(89, 233, 22, .8);
  border-radius: 30px;
  
}
.label-settings{
  padding: 0px 20px;
}
.input-settings{
  background-color: rgba(89, 233, 22, 0.645);
  border-radius: 999px;
  color: white;
  box-shadow:  0 0 8px rgba(89, 233, 22, .8);
  padding: 8px 10px;
  backdrop-filter: blur(13px);
  transition: border-color .18s,box-shadow .18s;
  
  margin:15px 15px;
}
.input-settings:focus{
  outline: none;
  border-color:  rgba(11, 226, 58, 0.9);
 box-shadow:  0 0 8px rgba(89, 233, 22, .18);

}
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
.bar.blur{
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
   gap: 8px;
   background: rgba(0,0,0,.7);
   z-index: 10;
 
}
.settings-main{
 display: flex;
   flex-direction: column;
   align-items: center;
     max-height: 80vh;
    
  overflow-y: auto;
}
.settings_btn{
  margin-left: 20px;
  width: 40px;
  border-radius: 999px;
  background-color: rgba(255, 255, 255, 0.308);
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
