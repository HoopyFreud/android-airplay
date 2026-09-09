<script lang="ts">
  import "../app.css";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Spinner } from "$lib/components/ui/spinner/index.js";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import * as InputOTP from "$lib/components/ui/input-otp/index.js";
  import * as Table from "$lib/components/ui/table/index.js";

  import { invoke } from "@tauri-apps/api/core";
  import {
    startService,
    stopService,
    isServiceRunning,
    getLifecycleStatus,
    onPluginEvent,
    getPlatformCapabilities,
    configureRecovery,
    validateBackgroundServiceSetup,
    normalizeBackgroundServiceError,
  } from 'tauri-plugin-background-service';

  interface Device {
    device_index: number,
    device_name: string,
    device_id: string,
    device_ip: string,
    device_requires_pin: boolean
  }
  
  interface DeviceListResponse {
    DeviceList: Device[]
  }
  
  interface DeviceResponse {
    Device: Device
  }
  
  interface ErrorResponse {
    Error: string
  }

  type ClientResponse = DeviceListResponse | DeviceResponse | ErrorResponse

  var device_list: Device[] = $state([])

  var err_msg: string | null = $state(null)

  async function getclients() {
    try {
      await invoke<ClientResponse>('get_client_list', { timeout: 5 }).then((reply) => {
        if ("DeviceList" in reply) {
          device_list = reply.DeviceList
          err_msg = null
        }
      })
    }
    catch (error) {
      err_msg = JSON.stringify(error as Object)
    }
  }

  async function connectClientIndex(index: number) {
    try {
      await invoke<ClientResponse>('connect_to_client_index', { index: index }).then((reply) => {
        if ("DeviceList" in reply) {
          device_list = reply.DeviceList
          err_msg = null
        }
      })
    }
    catch (error) {
      err_msg = JSON.stringify(error as Object)
    }
  }

  async function requestPIN(ip_address:string) {
		const response = await fetch('http://'+ip_address+'/pair-pin-start', {
			method: 'POST'
		});
	}
  /*
  // Query platform capabilities (call early to set UI expectations)
  const caps = await getPlatformCapabilities();
  console.log(caps.backgroundExecution);  // 'guaranteed' | 'bestEffort' | 'unsupported'
  console.log(caps.survivesReboot);       // 'guaranteed' | 'bestEffort' | 'unsupported'

  // Start the service (optionally configure the Android notification label)
  await startService({ serviceLabel: 'Casting via Airplay', foregroundServiceType: "mediaProjection" });

  // Check if running (simple boolean)
  const running = await isServiceRunning();

  // Query detailed service state
  const status = await getLifecycleStatus();
  console.log(status.state); // 'idle' | 'initializing' | 'running' | 'stopped'
  console.log(status.lastError); // null or error message
  console.log(status.desiredRunning); // true | false | undefined

  // Listen to lifecycle events
  const unlisten = await onPluginEvent((event) => {
    switch (event.type) {
      case 'started':
        console.log('Service started');
        break;
      case 'stopped':
        console.log('Service stopped:', event.reason);
        break;
      case 'error':
        console.error('Service error:', event.message);
        break;
    }
  });

  // Validate your platform setup (checks permissions, manifest entries)
  const report = await validateBackgroundServiceSetup();
  if (!report.ok) {
    for (const err of report.errors) {
      console.error(`[${err.code}] ${err.message}`);
      if (err.fix) console.error(`  Fix: ${err.fix}`);
    }
  }

  // Stop the service
  await stopService();

  // Clean up listener
  unlisten();*/

  let name = $state("");
  let greetMsg = $state("");

  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }
</script>

<div id="app" class="relative min-h-dvh">
    <h1>Connect to Airplay</h1>
    <div class="flex flex-row w-full gap-2 justify-center my-5">
      <Button type="button" onclick={getclients}>Refresh client list</Button>
    </div>
    <Table.Root>
      <Table.Body>
      
        {#each device_list as device}
          <Table.Row>
            <Table.Cell>{device.device_index}</Table.Cell>
            <Table.Cell>{device.device_name}</Table.Cell>
            <Table.Cell>{device.device_id}</Table.Cell>
            <Table.Cell>{device.device_ip}</Table.Cell>
            <Table.Cell>{device.device_requires_pin}</Table.Cell>
            <Table.Cell>
              {#if device.device_requires_pin}
                <Dialog.Root>
                    <Dialog.Trigger type="button">Connect</Dialog.Trigger>
                    <Dialog.Content class="sm:max-w-[425px]">
                      <Dialog.Header />
                      <div class="flex flex-col gap-3 items-center">
                        <Button class="w-fit" type="button" onclick={() => getClientPin(device.device_index)}>Request PIN</Button>
                        <InputOTP.Root maxlength={4} class="w-fit">
                          {#snippet children({ cells })}
                            <InputOTP.Group>
                              {#each cells as cell (cell)}
                                <InputOTP.Slot {cell} />
                              {/each}
                            </InputOTP.Group>
                          {/snippet}
                        </InputOTP.Root>
                        <Button class="w-fit" type="button" onclick={() => connectClientIndexPin(device.device_index)}>Connect</Button>
                      </div>
                      <Dialog.Footer>
                        <Dialog.Close type="button">Cancel</Dialog.Close>
                      </Dialog.Footer>
                    </Dialog.Content>
                </Dialog.Root>
              {:else}
                <Button type="button" onclick={() => connectClientIndex(device.device_index)}>Connect</Button>
              {/if}
            </Table.Cell>
          </Table.Row>
        {/each}
      </Table.Body>
    </Table.Root>
    
    {#if err_msg}
      Error: {err_msg}
    {/if}
</div>