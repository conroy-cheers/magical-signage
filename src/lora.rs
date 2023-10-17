use embassy_lora::LoraTimer;
use embassy_rp::clocks::RoscRng;

use embassy_time::Delay;
use embedded_hal_async::spi::SpiBus;
use lora_phy::mod_params::*;
use lora_phy::mod_traits::InterfaceVariant;
use lora_phy::sx1261_2::SX1261_2;
use lora_phy::LoRa;
use lorawan::default_crypto::DefaultFactory;
use lorawan::default_crypto::DefaultFactory as Crypto;
use lorawan_device::async_device::lora_radio::LoRaRadio;
use lorawan_device::async_device::{region, Device};

const LORAWAN_REGION: region::Region = region::Region::AU915;

type RadioType<SPI, IV> = Device<
    LoRaRadio<SX1261_2<SPI, IV>, Delay>,
    DefaultFactory,
    LoraTimer,
    RoscRng,
>;

pub(crate) async fn config_lora<SPI, IV>(spi: SPI, iv: IV) -> Result<RadioType<SPI, IV>, RadioError>
where
    SPI: SpiBus<u8>,
    IV: InterfaceVariant,
{
    let lora = LoRa::new(
        SX1261_2::new(BoardType::RpPicoWaveshareSx1262, spi, iv),
        true,
        Delay,
    )
    .await?;

    let radio = LoRaRadio::new(lora);
    let mut region: region::Configuration = region::Configuration::new(LORAWAN_REGION);
    region.set_receive_delay1(6500);
    let device: Device<_, Crypto, _, _> =
        Device::new(region, radio, LoraTimer::new(), RoscRng);

    Ok(device)
}
