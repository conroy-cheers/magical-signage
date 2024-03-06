use embassy_rp::clocks::RoscRng;
use embassy_time::Delay;
use embedded_hal_async::spi::SpiDevice;
use lora_phy::lorawan_radio::LorawanRadio;
use lora_phy::mod_traits::InterfaceVariant;
use lora_phy::sx126x::Sx126x;
use lora_phy::LoRa;
use lora_phy::{mod_params::*, sx126x};
use lorawan::default_crypto::DefaultFactory;
use lorawan_device::async_device::{region, Device, EmbassyTimer};

const LORAWAN_REGION: region::Region = region::Region::AU915;
const MAX_TX_POWER: u8 = 14;

type RadioType<SPI, IV> = Device<
    LorawanRadio<Sx126x<SPI, IV>, Delay, MAX_TX_POWER>,
    DefaultFactory,
    EmbassyTimer,
    RoscRng,
>;

pub(crate) async fn config_lora<SPI, IV>(spi: SPI, iv: IV) -> Result<RadioType<SPI, IV>, RadioError>
where
    SPI: SpiDevice,
    IV: InterfaceVariant,
{
    let config = sx126x::Config {
        chip: sx126x::Sx126xVariant::Sx1262,
        tcxo_ctrl: Some(sx126x::TcxoCtrlVoltage::Ctrl1V7),
        use_dcdc: true,
        use_dio2_as_rfswitch: true,
    };
    let lora = LoRa::new(Sx126x::new(spi, iv, config), true, Delay).await?;

    let radio: LorawanRadio<_, _, MAX_TX_POWER> = lora.into();
    let region: region::Configuration = region::Configuration::new(LORAWAN_REGION);
    // region.set_receive_delay1(6500);
    let device: Device<
        LorawanRadio<Sx126x<SPI, IV>, Delay, 14>,
        DefaultFactory,
        EmbassyTimer,
        RoscRng,
    > = Device::new(region, radio, EmbassyTimer::new(), RoscRng);

    Ok(device)
}
