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
  <div  class="container">

      <div class="dragbar" data-tauri-drag-region></div>
    <div ref="listEl" class="list" :class="{blur:showSettings}">
      <div v-if="items.length === 0" class="hint">等待字幕… 按 Start 开始</div>
      <div v-for="it in items" :key="it.id" class="row">
        <div class="src">{{ it.text }}</div>
        <div class="dst">{{ it.trans }}</div>
      </div>
    </div >
   <div v-if="showSettings" class="settings">
    <div class="settings-main">
      
     <label class="label-settings">API 端点    <el-input  v-model="from.ai_api_base" ></el-input></label>
     <label class="label-settings">模型 ID<el-input v-model="from.ai_model" /></label> 
     <label class="label-settings" >API Key <el-input show-password="true" v-model="from.ai_api_key" type="password" /></label>
    <label class="label-settings">目标语言<el-input v-model="from.target_lang"/></label> 
     <label class="label-settings">最大切分时长<el-input v-model.number="from.vad_max_speech_ms"><template #append>ms</template></el-input></label>
    <label class="label-settings">静音时超时切分时长<el-input v-model.number="from.vad_slience_ms"><template #append>ms</template></el-input></label> 
    </div>
          <div class="list-button">
               <el-button round @click="save_c()">save</el-button>
              <el-button round  @click="showSettings =false">exit</el-button>
              <el-button round @click="openWindow()">{{ show? "关闭标题栏":"开启标题栏" }}</el-button>
         <el-button round  @click="cleanitem()">清理字幕</el-button>
       <el-button round @click="exportText()">导出字幕</el-button>
      
        
         </div>
        </div>
    <div class="bar" :class="{blur:showSettings}">
        <el-button  @click="openSettings" circle >⚙</el-button>
    <button @click="toggle" class="btn" :disabled="busy" :class="{stop: running}">{{ running? "stop":"start" }}</button>
    </div>
  </div>
</template>

<style>
.bar .el-button{
  margin-left: 10px;
  background-color: #ffffff10;
  border: none;
}
.bar .el-button:hover{
  background-color: #08080852;
  color: #59e916;
}


.dragbar { height: 22px; flex-shrink: 0; cursor: move; }
html, body { margin: 0; }
.list-button{
  margin-top: 13px;
  display: flex;
  padding: 0px 15px;
  gap: 15px;
  justify-content: center;
overflow: visible;

  
}
.list-button .el-button{
  background-color: #09ff0062;
  color: rgba(255, 255, 255, 0.932);
  box-shadow:  0 0 8px rgba(89, 233, 22, .8);
}
.list-button .el-button:hover{
  background-color: #11111162;
  backdrop-filter: blur(12px);
  box-shadow:  0 0 10px rgb(89, 233, 22);
  color: white;
}
.settings .el-input-group__append {
  background: transparent;
  color: rgba(255, 255, 255, .6);
}


.label-settings{
  
  padding: 0px 20px;
  display: flex;
  flex-direction: column;
   margin-bottom: 10px;
  gap: 4px  ;
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
.settings .el-input {
 --el-input-bg-color: rgba(255, 255, 255, .08);
  --el-input-text-color: #fff;
  --el-input-border-color: rgba(255, 255, 255, .2);
  --el-input-hover-border-color: rgba(255, 255, 255, .45);
  --el-input-focus-border-color: #59e916;  /* 对焦时用你的荧光绿 */
  --el-input-placeholder-color: rgba(255, 255, 255, .35);
}
.settings {
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
