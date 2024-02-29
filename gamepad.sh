mount none /configs -t configfs
cd /configs/usb_gadget/

mkdir -p LinTxGamePad
cd LinTxGamePad


echo 0x1d6b > idVendor # Linux Foundation
echo 0x0104 > idProduct # Multifunction Composite Joystick Gadget
echo 0x0100 > bcdDevice # v1.0.0
echo 0x0200 > bcdUSB # USB2
echo 0x02 > bDeviceClass
echo 0x00 > bDeviceSubClass
echo 0x00 > bDeviceProtocol

# Perform localization
mkdir -p strings/0x409

echo "0123456789" > strings/0x409/serialnumber
echo "LinTx" > strings/0x409/manufacturer
echo "LinTx GamePad" > strings/0x409/product

# Define the functions of the device
mkdir functions/hid.usb0
echo 0 > functions/hid.usb0/protocol
echo 0 > functions/hid.usb0/subclass
echo 5 > functions/hid.usb0/report_length

# Write report descriptor ( X and Y analog joysticks plus 8 buttons )
#echo "05010904A1011581257F0901A10009300931750895028102C005091901290815002501750195088102C0" | xxd -r -ps > functions/hid.usb0/report_desc

echo "05 01 09 05 A1 01 A1 00 05 09 19 01 29 08 15 00 25 01 95 08 75 01 81 02 05 01 09 30 09 31 09 32 09 33 15 81 25 7F 75 08 95 04 81 02 C0 C0 " | xxd -r -ps > functions/hid.usb0/report_desc
# Create configuration file
mkdir configs/c.1
mkdir configs/c.1/strings/0x409

echo 0x80 > configs/c.1/bmAttributes
echo 100 > configs/c.1/MaxPower # 100 mA
echo "LinTx GamePad Configuration" > configs/c.1/strings/0x409/configuration

# Link the configuration file
ln -s functions/hid.usb0 configs/c.1

ls /sys/class/udc > UDC

echo peripheral > /sys/devices/platform/soc/1c19000.usb/musb-hdrc.2.auto/mode

